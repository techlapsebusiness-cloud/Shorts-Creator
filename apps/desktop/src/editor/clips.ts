import type { Clip } from "../bindings/Clip";
import type { Project } from "../bindings/Project";
import type { TrackId } from "../bindings/TrackId";

export function screenClips(project: Project): Clip[] {
  return (
    project.tracks.find((track) => track.id === "V1")?.clips.filter(isMedia) ??
    []
  );
}

export function splitClip(clip: Clip, at: number): Clip[] | null {
  const body = mediaBody(clip);
  if (!body) {
    return null;
  }
  if (
    at <= clip.timelineStart + 0.02 ||
    at >= clip.timelineStart + clip.duration - 0.02
  ) {
    return null;
  }
  const local = at - clip.timelineStart;
  const speed = body.speed || 1;
  const cut = body.sourceIn + local * speed;
  return [
    {
      ...clip,
      id: `${clip.id}-a`,
      duration: local,
      body: { ...body, sourceOut: cut },
    },
    {
      ...clip,
      id: `${clip.id}-b`,
      timelineStart: at,
      duration: clip.duration - local,
      body: { ...body, sourceIn: cut },
    },
  ];
}

export function trimClip(clip: Clip, duration: number): Clip | null {
  const body = mediaBody(clip);
  if (!body || duration <= 0.02 || duration >= clip.duration) {
    return null;
  }
  const speed = body.speed || 1;
  return {
    ...clip,
    duration,
    body: { ...body, sourceOut: body.sourceIn + duration * speed },
  };
}

export function speedClip(clip: Clip, speed: number): Clip | null {
  const body = mediaBody(clip);
  if (!body) {
    return null;
  }
  const next = Math.min(4, Math.max(0.25, speed));
  const source = Math.max(0.04, body.sourceOut - body.sourceIn);
  return {
    ...clip,
    duration: source / next,
    body: { ...body, speed: next },
  };
}

export function replaceClip(
  project: Project,
  trackId: TrackId,
  clipId: string,
  next: Clip[],
): Project {
  return {
    ...project,
    tracks: project.tracks.map((track) => {
      if (track.id !== trackId) {
        return track;
      }
      const index = track.clips.findIndex((clip) => clip.id === clipId);
      if (index < 0) {
        return track;
      }
      const clips = [...track.clips];
      clips.splice(index, 1, ...next);
      return { ...track, clips };
    }),
  };
}

export function rippleClip(
  project: Project,
  trackId: TrackId,
  clipId: string,
): Project {
  return {
    ...project,
    tracks: project.tracks.map((track) => {
      if (track.id !== trackId) {
        return track;
      }
      const clip = track.clips.find((item) => item.id === clipId);
      if (!clip) {
        return track;
      }
      const end = clip.timelineStart + clip.duration;
      return {
        ...track,
        clips: track.clips
          .filter((item) => item.id !== clipId)
          .map((item) =>
            item.timelineStart >= end
              ? { ...item, timelineStart: item.timelineStart - clip.duration }
              : item,
          ),
      };
    }),
  };
}

export function cutRange(
  project: Project,
  start: number,
  end: number,
): Project {
  return {
    ...project,
    tracks: project.tracks.map((track) => {
      if (track.id !== "V1" && track.id !== "A1") {
        return track;
      }
      const clips: Clip[] = [];
      for (const clip of track.clips) {
        const body = mediaBody(clip);
        const clipEnd = clip.timelineStart + clip.duration;
        if (!body || start < clip.timelineStart || end > clipEnd) {
          clips.push(clip);
          continue;
        }
        const speed = body.speed || 1;
        const head = start - clip.timelineStart;
        if (head > 0.02) {
          clips.push({
            ...clip,
            id: `${clip.id}-a`,
            duration: head,
            body: { ...body, sourceOut: body.sourceIn + head * speed },
          });
        }
        const tail = clipEnd - end;
        if (tail > 0.02) {
          clips.push({
            ...clip,
            id: `${clip.id}-b`,
            timelineStart: start,
            duration: tail,
            body: {
              ...body,
              sourceIn: body.sourceIn + (end - clip.timelineStart) * speed,
            },
          });
        }
      }
      return { ...track, clips };
    }),
  };
}

export function mediaSource(clip: Clip | undefined): string {
  if (!clip) {
    return "";
  }
  const body = clip.body;
  if (body.kind === "video" || body.kind === "audio") {
    return body.source;
  }
  return "";
}

export function mediaRange(clip: Clip | undefined): {
  start: number;
  end: number;
} {
  if (!clip) {
    return { start: 0, end: 0 };
  }
  const body = clip.body;
  if (body.kind === "video" || body.kind === "audio") {
    return { start: body.sourceIn, end: body.sourceOut };
  }
  return { start: 0, end: clip.duration };
}

function isMedia(clip: Clip) {
  return clip.body.kind === "video" || clip.body.kind === "audio";
}

function mediaBody(clip: Clip) {
  if (clip.body.kind === "video" || clip.body.kind === "audio") {
    return clip.body;
  }
  return null;
}
