import { describe, expect, it } from "vitest";
import type { Project } from "../bindings/Project";
import { replaceClip, screenClips, splitClip } from "./clips";

const project = {
  version: 1,
  id: "1000",
  number: 1,
  name: "Take 1000",
  slug: "take-1000",
  tracks: [
    { id: "V3", clips: [], muted: false, solo: false },
    { id: "V2", clips: [], muted: false, solo: false },
    {
      id: "V1",
      muted: false,
      solo: false,
      clips: [
        {
          id: "clip-screen",
          timelineStart: 0,
          duration: 2,
          keyframes: [],
          body: {
            kind: "video",
            source: "/takes/1000/screen.mov",
            sourceIn: 0,
            sourceOut: 2,
            speed: 1,
          },
        },
      ],
    },
  ],
  layout: {
    preset: "screenCam",
    camPosition: "bottomRight",
    camSize: "M",
    camShape: "circle",
    safeZone: true,
    accent: "#FF7A3D",
    transition: "cut",
    markersSecs: [],
  },
  audioChain: {} as Project["audioChain"],
  assets: {} as Project["assets"],
  export: {} as Project["export"],
  ai: {} as Project["ai"],
  sync: { screenOffsetMs: 0, camOffsetMs: 0, micOffsetMs: 0 },
} satisfies Project;

describe("editor clips", () => {
  it("splits a clip at the playhead and keeps the source range", () => {
    const clip = screenClips(project)[0];
    const parts = splitClip(clip, 0.5);
    expect(parts).not.toBeNull();
    const [left, right] = parts ?? [];
    expect(left.body.kind === "video" && left.body.sourceOut).toBe(0.5);
    expect(right.body.kind === "video" && right.body.sourceIn).toBe(0.5);
    const next = replaceClip(project, "V1", clip.id, parts ?? []);
    expect(screenClips(next)).toHaveLength(2);
  });
});
