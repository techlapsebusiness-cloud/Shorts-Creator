import { convertFileSrc } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import {
  cutRange,
  mediaRange,
  mediaSource,
  replaceClip,
  rippleClip,
  screenClips,
  speedClip,
  splitClip,
  trimClip,
} from "../editor/clips";
import { useSession } from "../session";
import { Card } from "../components/Card";
import { Pill } from "../components/Pill";
import { SelectField } from "../components/Field";
import { Toggle } from "../components/Toggle";
import {
  camRect,
  captionPage,
  captionRect,
  cropTarget,
  safeRect,
  stepCrop,
  zoomAt,
  type CamSlot,
  type CropFollow,
  type Rect,
} from "../layout/frame";
import {
  historyOf,
  nextTransition,
  pushHistory,
  redoHistory,
  rippleDelete,
  setSpeed,
  splitSpan,
  trimSpan,
  undoHistory,
  waveformPeaks,
  type History,
  type Span,
  type TransitionName,
} from "../timeline/edit";
import styles from "./screens.module.css";

const POSITIONS = [
  "Top left",
  "Top",
  "Top right",
  "Left",
  "Center",
  "Right",
  "Bottom left",
  "Bottom",
  "Bottom right",
] as const;

const FRAME_W = 342;
const FRAME_H = 608;
const SOURCE_W = 1920;
const SOURCE_H = 1080;
const HINGLISH = [
  "Yeh",
  "file",
  "crash",
  "ke",
  "baad",
  "bhi",
  "survive",
  "karti",
  "hai",
  "kyunki",
  "rename",
  "atomic",
  "hota",
  "hai",
];
const ENGLISH = [
  "This",
  "file",
  "survives",
  "a",
  "crash",
  "because",
  "the",
  "rename",
  "is",
  "atomic",
];

const TOOLS = [
  "Select (V)",
  "Split (S)",
  "Trim (T)",
  "Ripple delete",
  "Speed (R)",
  "Zoom (Z)",
  "Marker (M)",
  "Transition (Shift T)",
] as const;

const TRACKS = [
  ["V3 FX", ""],
  ["V2 CAM", styles.camera],
  ["V1 SCREEN", styles.screen],
  ["TXT", styles.text],
  ["A1 VOICE", styles.voice],
  ["A2 MUSIC", styles.music],
  ["A3 SFX", styles.sfx],
] as const;

const CAM_POSITION = [
  "topLeft",
  "topCenter",
  "topRight",
  "middleLeft",
  "middleCenter",
  "middleRight",
  "bottomLeft",
  "bottomCenter",
  "bottomRight",
] as const;

export function EditScreen() {
  const session = useSession();
  const project = session.project;
  const screenVideo = useRef<HTMLVideoElement>(null);
  const camVideo = useRef<HTMLVideoElement>(null);
  const [playing, setPlaying] = useState(false);
  const [playhead, setPlayhead] = useState(0);
  const [tab, setTab] = useState<"Layout" | "Captions" | "Motion">("Layout");
  const [position, setPosition] = useState<CamSlot>("Bottom right");
  const [size, setSize] = useState<"S" | "M" | "L">("M");
  const [shape, setShape] = useState<"Circle" | "Rounded" | "Square">("Circle");
  const [safeZone, setSafeZone] = useState(true);
  const [autoZoom, setAutoZoom] = useState(true);
  const [words, setWords] = useState(true);
  const [language, setLanguage] = useState("Hinglish");
  const [tool, setTool] = useState("Select (V)");
  const [snap, setSnap] = useState(true);
  const [track, setTrack] = useState<History<Span[]>>(() =>
    historyOf([{ id: "screen", startMs: 0, durationMs: 10_000 }]),
  );
  const cropRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    const node = cropRef.current;
    if (!node) {
      return;
    }
    const place = (rect: Rect, spaceW: number, spaceH: number) => {
      node.style.left = `${(rect.x / spaceW) * 100}%`;
      node.style.top = `${(rect.y / spaceH) * 100}%`;
      node.style.width = `${(rect.width / spaceW) * 100}%`;
      node.style.height = `${(rect.height / spaceH) * 100}%`;
    };
    if (!autoZoom) {
      place(
        cropTarget(SOURCE_W / 2, SOURCE_H / 2, SOURCE_W, SOURCE_H, 1),
        SOURCE_W,
        SOURCE_H,
      );
      return;
    }
    const centered = cropTarget(
      SOURCE_W / 2,
      SOURCE_H / 2,
      SOURCE_W,
      SOURCE_H,
      1,
    );
    let follow: CropFollow = { x: centered.x, y: centered.y, vx: 0, vy: 0 };
    const started = performance.now();
    let frame = 0;
    const tick = (now: number) => {
      const elapsed = (now - started) / 1000;
      const cursorX =
        SOURCE_W * (0.25 + 0.5 * (0.5 + 0.5 * Math.sin(elapsed * 0.7)));
      const cursorY =
        SOURCE_H * (0.35 + 0.3 * (0.5 + 0.5 * Math.cos(elapsed * 0.45)));
      const target = cropTarget(
        cursorX,
        cursorY,
        SOURCE_W,
        SOURCE_H,
        zoomAt(
          [
            [0, 1],
            [4, 1.4],
          ],
          elapsed % 8,
        ),
      );
      const stepped = stepCrop(follow, target, SOURCE_W, SOURCE_H, 1 / 30);
      follow = stepped.follow;
      place(stepped.rect, SOURCE_W, SOURCE_H);
      frame = window.requestAnimationFrame(tick);
    };
    frame = window.requestAnimationFrame(tick);
    return () => window.cancelAnimationFrame(frame);
  }, [autoZoom]);
  const [markers, setMarkers] = useState(0);
  const [zooms, setZooms] = useState(0);
  const [transition, setTransition] = useState<TransitionName>("cut");
  const peaks = waveformPeaks([0.2, 0.55, 0.9, 0.4, 0.7, 0.35, 0.8, 0.5], 8);

  const openedClips = project ? screenClips(project) : [];
  const activeClip =
    openedClips.find(
      (clip) =>
        playhead >= clip.timelineStart &&
        playhead <= clip.timelineStart + clip.duration,
    ) ?? openedClips.at(-1);

  function splitScreen() {
    if (project && activeClip) {
      const at =
        playhead > activeClip.timelineStart &&
        playhead < activeClip.timelineStart + activeClip.duration
          ? playhead
          : activeClip.timelineStart + activeClip.duration / 2;
      const parts = splitClip(activeClip, at);
      if (parts) {
        session.updateProject(replaceClip(project, "V1", activeClip.id, parts));
      }
      return;
    }
    const target = track.present.at(-1);
    if (!target || target.durationMs < 2) {
      return;
    }
    const at = target.startMs + Math.floor(target.durationMs / 2);
    const next = splitSpan(track.present, target.id, at);
    if (next !== track.present) {
      setTrack(pushHistory(track, next));
    }
  }

  function selectedSpan() {
    return track.present.at(-1);
  }

  function commit(next: Span[]) {
    if (next !== track.present) {
      setTrack(pushHistory(track, next));
    }
  }

  function trimSelected() {
    if (project && activeClip) {
      const trimmed = trimClip(activeClip, activeClip.duration / 2);
      if (trimmed) {
        session.updateProject(
          replaceClip(project, "V1", activeClip.id, [trimmed]),
        );
      }
      return;
    }
    const target = selectedSpan();
    if (!target || target.durationMs < 2) {
      return;
    }
    commit(
      trimSpan(track.present, target.id, Math.floor(target.durationMs / 2)),
    );
  }

  function speedSelected() {
    if (project && activeClip) {
      const body = activeClip.body;
      const current =
        body.kind === "video" || body.kind === "audio" ? body.speed : 1;
      const next = speedClip(activeClip, current >= 2 ? 1 : 2);
      if (next) {
        session.updateProject(
          replaceClip(project, "V1", activeClip.id, [next]),
        );
      }
      return;
    }
    const target = selectedSpan();
    if (!target) {
      return;
    }
    const current = target.speed ?? 1;
    commit(setSpeed(track.present, target.id, current >= 2 ? 1 : 2));
  }

  function rippleSelected() {
    if (project && activeClip) {
      session.updateProject(rippleClip(project, "V1", activeClip.id));
      return;
    }
    const target = track.present.at(-1);
    if (!target) {
      return;
    }
    const next = rippleDelete(track.present, target.id);
    if (next !== track.present) {
      setTrack(pushHistory(track, next));
    }
  }

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      const target = event.target;
      if (
        target instanceof HTMLElement &&
        (target.tagName === "INPUT" ||
          target.tagName === "SELECT" ||
          target.tagName === "TEXTAREA")
      ) {
        return;
      }
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "z") {
        event.preventDefault();
        setTrack((current) =>
          event.shiftKey ? redoHistory(current) : undoHistory(current),
        );
        return;
      }
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }
      const key = event.key.toLowerCase();
      if (key === "s") {
        splitScreen();
      } else if (key === "t" && event.shiftKey) {
        setTransition((current) => nextTransition(current));
      } else if (key === "t") {
        trimSelected();
      } else if (key === "r") {
        speedSelected();
      } else if (key === "m") {
        setMarkers((count) => count + 1);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [track, project, playhead]);

  const captionClip = project?.tracks.find((track) => track.id === "TXT")
    ?.clips[0];
  const transcript =
    captionClip?.body.kind === "caption" ? captionClip.body.words : [];
  const spoken =
    transcript.length > 0
      ? transcript.map((word) => word.text)
      : language === "English"
        ? ENGLISH
        : HINGLISH;
  const page = captionPage(spoken, language === "English" ? 2 : 9);
  const cam = camRect(FRAME_W, FRAME_H, position, size, safeZone);
  const caption = captionRect(
    FRAME_W,
    FRAME_H,
    words ? page.lines.length : 1,
    safeZone,
  );
  const safe = safeRect(FRAME_W, FRAME_H);
  const shownLines = words ? page.lines : [page.lines.flat()];

  return (
    <div className={styles.page}>
      <div className={styles.columns}>
        <div className={styles.stack}>
          <Card title="Media">
            {project ? (
              <>
                <p className={styles.meta}>
                  {fileName(mediaSource(openedClips[0]))}
                </p>
                <p className={styles.meta}>
                  {fileName(
                    mediaSource(
                      project.tracks.find((item) => item.id === "V2")?.clips[0],
                    ),
                  )}
                </p>
                <p className={styles.meta}>
                  {fileName(
                    mediaSource(
                      project.tracks.find((item) => item.id === "A1")?.clips[0],
                    ),
                  )}
                </p>
              </>
            ) : (
              <>
                <p className={styles.meta}>screen.mov</p>
                <p className={styles.meta}>cam.mov</p>
                <p className={styles.meta}>mic.wav</p>
                <p className={styles.meta}>Open a recording to edit it.</p>
              </>
            )}
          </Card>
          <Card title="Chapters">
            <p className={styles.meta}>0:00 Hook</p>
            <p className={styles.meta}>0:08 Demo</p>
            <p className={styles.meta}>0:15 Fix</p>
          </Card>
        </div>
        <div className={styles.stack}>
          <p className={styles.kicker}>342 x 608 at fit</p>
          <div className={styles.previewWrap}>
            <div className={styles.preview}>
              {mediaSource(openedClips[0]) ? (
                <video
                  ref={screenVideo}
                  className={styles.film}
                  src={fileSrc(playbackPath(mediaSource(openedClips[0])))}
                  onError={(event) => {
                    const node = event.currentTarget;
                    const original = fileSrc(mediaSource(openedClips[0]));
                    if (node.src !== original) {
                      node.src = original;
                    }
                  }}
                  onTimeUpdate={(event) => {
                    const node = event.currentTarget;
                    const range = mediaRange(activeClip);
                    setPlayhead(node.currentTime);
                    if (
                      camVideo.current &&
                      Math.abs(
                        camVideo.current.currentTime - node.currentTime,
                      ) > 0.2
                    ) {
                      camVideo.current.currentTime = node.currentTime;
                    }
                    if (
                      node.currentTime >= range.end &&
                      range.end > range.start
                    ) {
                      node.pause();
                      setPlaying(false);
                    }
                  }}
                  playsInline
                />
              ) : (
                <div ref={cropRef} className={styles.crop} />
              )}
              {mediaSource(
                project?.tracks.find((item) => item.id === "V2")?.clips[0],
              ) ? (
                <video
                  ref={camVideo}
                  className={styles.film}
                  style={{
                    ...percentBox(cam, FRAME_W, FRAME_H),
                    borderRadius:
                      shape === "Circle"
                        ? "50%"
                        : shape === "Rounded"
                          ? "18%"
                          : "2px",
                    objectFit: "cover",
                  }}
                  src={fileSrc(
                    mediaSource(
                      project?.tracks.find((item) => item.id === "V2")
                        ?.clips[0],
                    ),
                  )}
                  muted
                  playsInline
                />
              ) : null}
              {safeZone ? (
                <div
                  className={styles.safe}
                  style={percentBox(safe, FRAME_W, FRAME_H)}
                />
              ) : null}
              {mediaSource(
                project?.tracks.find((item) => item.id === "V2")?.clips[0],
              ) ? null : (
                <div
                  className={styles.cam}
                  data-shape={shape}
                  style={{
                    ...percentBox(cam, FRAME_W, FRAME_H),
                    borderRadius:
                      shape === "Circle"
                        ? "50%"
                        : shape === "Rounded"
                          ? "18%"
                          : "2px",
                  }}
                />
              )}
              <p
                className={styles.caption}
                style={percentBox(caption, FRAME_W, FRAME_H)}
              >
                {shownLines.map((line, lineIndex) => (
                  <span key={line.join("-")} className={styles.captionLine}>
                    {line.map((word, wordIndex) => {
                      const stamp = transcript[lineIndex * 8 + wordIndex];
                      return (
                        <button
                          key={`${word}-${wordIndex}`}
                          type="button"
                          className={
                            words &&
                            lineIndex === page.activeLine &&
                            wordIndex === page.activeIndex
                              ? styles.word
                              : styles.tool
                          }
                          onClick={() => {
                            if (project && stamp) {
                              session.updateProject(
                                cutRange(project, stamp.start, stamp.end),
                              );
                            }
                          }}
                        >
                          {word}{" "}
                        </button>
                      );
                    })}
                  </span>
                ))}
              </p>
            </div>
          </div>
          <Toggle
            label="Shorts safe zone"
            checked={safeZone}
            onChange={setSafeZone}
          />
        </div>
        <Card title="Inspector">
          <div className={styles.row}>
            {(["Layout", "Captions", "Motion"] as const).map((item) => (
              <Pill
                key={item}
                selected={tab === item}
                onClick={() => setTab(item)}
              >
                {item}
              </Pill>
            ))}
          </div>
          <p className={styles.kicker}>Cam position</p>
          <div className={styles.grid} role="group" aria-label="Cam position">
            {POSITIONS.map((item) => (
              <button
                key={item}
                type="button"
                className={styles.cell}
                aria-label={item}
                aria-pressed={position === item}
                onClick={() => {
                  setPosition(item);
                  if (project) {
                    session.updateProject({
                      ...project,
                      layout: {
                        ...project.layout,
                        camPosition: CAM_POSITION[POSITIONS.indexOf(item)],
                      },
                    });
                  }
                }}
              />
            ))}
          </div>
          <div className={styles.row}>
            {["S", "M", "L"].map((item) => (
              <Pill
                key={item}
                selected={size === item}
                onClick={() => setSize(item as "S" | "M" | "L")}
              >
                {item}
              </Pill>
            ))}
          </div>
          <div className={styles.row}>
            {["Circle", "Rounded", "Square"].map((item) => (
              <Pill
                key={item}
                selected={shape === item}
                onClick={() =>
                  setShape(item as "Circle" | "Rounded" | "Square")
                }
              >
                {item}
              </Pill>
            ))}
          </div>
          <Toggle
            label="Background removal"
            checked={false}
            disabled
            onChange={() => undefined}
          />
          <p className={styles.meta}>
            Shapes only. A portrait model is not in this build.
          </p>
          <Toggle
            label="Auto-zoom on cursor"
            checked={autoZoom}
            onChange={setAutoZoom}
          />
          <Toggle
            label="Word-by-word captions"
            checked={words}
            onChange={setWords}
          />
          <SelectField
            label="Caption language"
            value={language}
            options={["Hinglish", "English", "Hindi"]}
            onChange={setLanguage}
          />
        </Card>
      </div>
      <div className={styles.stack}>
        <div className={styles.row}>
          <button
            type="button"
            className={styles.tool}
            onClick={() => {
              const node = screenVideo.current;
              if (!node) {
                return;
              }
              if (node.paused) {
                const range = mediaRange(activeClip);
                if (
                  node.currentTime < range.start ||
                  node.currentTime >= range.end
                ) {
                  node.currentTime = range.start;
                }
                if (camVideo.current) {
                  camVideo.current.currentTime = node.currentTime;
                  void camVideo.current.play();
                }
                void node.play();
                setPlaying(true);
              } else {
                node.pause();
                camVideo.current?.pause();
                setPlaying(false);
              }
            }}
          >
            {playing ? "Pause" : "Play"}
          </button>
          <input
            type="range"
            min={0}
            max={Math.max(
              openedClips.reduce(
                (end, clip) =>
                  Math.max(end, clip.timelineStart + clip.duration),
                1,
              ),
              1,
            )}
            step={0.01}
            value={playhead}
            aria-label="Playhead"
            onChange={(event) => {
              const next = Number(event.target.value);
              setPlayhead(next);
              if (screenVideo.current) {
                screenVideo.current.currentTime = next;
              }
              if (camVideo.current) {
                camVideo.current.currentTime = next;
              }
            }}
          />
        </div>
        <div className={styles.tools}>
          {TOOLS.map((item) => (
            <button
              key={item}
              type="button"
              className={styles.tool}
              aria-pressed={tool === item}
              onClick={() => {
                setTool(item);
                if (item === "Split (S)") {
                  splitScreen();
                }
                if (item === "Trim (T)") {
                  trimSelected();
                }
                if (item === "Ripple delete") {
                  rippleSelected();
                }
                if (item === "Speed (R)") {
                  speedSelected();
                }
                if (item === "Zoom (Z)") {
                  setZooms((count) => count + 1);
                }
                if (item === "Marker (M)") {
                  setMarkers((count) => count + 1);
                  if (project) {
                    session.updateProject({
                      ...project,
                      layout: {
                        ...project.layout,
                        markersSecs: [...project.layout.markersSecs, playhead],
                      },
                    });
                  }
                }
                if (item === "Transition (Shift T)") {
                  const next = nextTransition(
                    (project?.layout.transition as TransitionName) ||
                      transition,
                  );
                  setTransition(next);
                  if (project) {
                    session.updateProject({
                      ...project,
                      layout: { ...project.layout, transition: next },
                    });
                  }
                }
              }}
            >
              {item}
            </button>
          ))}
          <Toggle label="Snapping" checked={snap} onChange={setSnap} />
          <label className={styles.meta}>
            Zoom
            <input
              type="range"
              min={1}
              max={8}
              defaultValue={2}
              aria-label="Timeline zoom"
            />
          </label>
        </div>
        <ul
          className={styles.tracks}
          aria-label="Timeline"
          data-markers={project?.layout.markersSecs.length ?? markers}
          data-zooms={zooms}
          data-transition={project?.layout.transition ?? transition}
        >
          {TRACKS.map(([name, clip]) => (
            <li key={name} className={styles.track}>
              <span className={styles.trackName}>{name}</span>
              <span
                className={styles.lane}
                data-testid={name === "V1 SCREEN" ? "v1-lane" : undefined}
                data-count={
                  name === "V1 SCREEN"
                    ? project
                      ? openedClips.length
                      : track.present.length
                    : undefined
                }
              >
                {name === "V1 SCREEN" ? (
                  (project ? openedClips : track.present).map((span) => (
                    <span
                      key={span.id}
                      className={`${styles.clip} ${styles.screen}`}
                    />
                  ))
                ) : name === "A1 VOICE" ? (
                  <span
                    className={`${styles.clip} ${styles.voice}`}
                    data-peaks={peaks.join(",")}
                    title="Voice waveform"
                  >
                    {peaks.map((peak, index) => (
                      <i
                        key={index}
                        className={styles.peak}
                        style={{
                          height: `${Math.max(2, Math.round(peak * 12))}px`,
                        }}
                      />
                    ))}
                  </span>
                ) : clip ? (
                  <span className={`${styles.clip} ${clip}`} />
                ) : null}
              </span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}

function fileName(path: string) {
  if (!path) {
    return "—";
  }
  const parts = path.split(/[/\\]/);
  return parts.at(-1) || path;
}

function playbackPath(path: string) {
  if (path.endsWith("screen.mov")) {
    return path.replace(/screen\.mov$/, "screen.720p.mp4");
  }
  return path;
}

function fileSrc(path: string) {
  if (!path || typeof convertFileSrc !== "function") {
    return path;
  }
  try {
    return convertFileSrc(path);
  } catch {
    return path;
  }
}

function percentBox(rect: Rect, spaceW: number, spaceH: number) {
  return {
    left: `${(rect.x / spaceW) * 100}%`,
    top: `${(rect.y / spaceH) * 100}%`,
    width: `${(rect.width / spaceW) * 100}%`,
    height: `${(rect.height / spaceH) * 100}%`,
  };
}
