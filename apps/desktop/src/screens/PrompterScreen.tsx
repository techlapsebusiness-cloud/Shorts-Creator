import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import {
  followOffsetPx,
  fuzzyMatch,
  nudgeOffset,
  scrollMode,
  scrollOffsetPx,
} from "../prompter/scroll";
import styles from "./PrompterScreen.module.css";

const FALLBACK = [
  "This file survives a crash.",
  "[CUE] point at the folder",
  "The edit lives in project.json.",
  "Build the next short.",
];

export function PrompterScreen() {
  const [lines, setLines] = useState(FALLBACK);
  const [speed, setSpeed] = useState(48);
  const [textSize, setTextSize] = useState(48);
  const [mirror, setMirror] = useState(false);
  const [opacity, setOpacity] = useState(0.92);
  const [underWebcam, setUnderWebcam] = useState(false);
  const [shown, setShown] = useState(true);
  const [paused, setPaused] = useState(false);
  const trackRef = useRef<HTMLDivElement>(null);
  const speedRef = useRef(speed);
  const pausedRef = useRef(paused);
  const nudgeRef = useRef(0);
  const runningRef = useRef(false);
  const startedAt = useRef<number | null>(null);
  const pausedMs = useRef(0);
  const pauseBegan = useRef<number | null>(null);
  const followRef = useRef<number | null>(null);

  speedRef.current = speed;
  pausedRef.current = paused;

  useEffect(() => {
    document.documentElement.style.background = "transparent";
    document.body.style.background = "transparent";
    void invoke<string[]>("prompter_lines")
      .then((loaded) => {
        if (loaded.length > 0) {
          setLines(loaded);
        }
      })
      .catch(() => undefined);
  }, []);

  useEffect(() => {
    let stop = false;
    async function poll() {
      while (!stop) {
        const running = await invoke<boolean>("prompter_running").catch(
          () => false,
        );
        runningRef.current = running;
        await new Promise((resolve) => setTimeout(resolve, 200));
      }
    }
    void poll();
    return () => {
      stop = true;
    };
  }, []);

  useEffect(() => {
    let frame = 0;
    const tick = (now: number) => {
      const scrolling = runningRef.current && !pausedRef.current;
      if (scrolling) {
        if (startedAt.current === null) {
          startedAt.current = now;
        }
        if (pauseBegan.current !== null) {
          pausedMs.current += now - pauseBegan.current;
          pauseBegan.current = null;
        }
        const elapsed = now - startedAt.current;
        const timed = scrollOffsetPx(
          elapsed,
          pausedMs.current,
          speedRef.current,
        );
        const offset =
          followRef.current === null
            ? nudgeRef.current + timed
            : followRef.current;
        if (trackRef.current) {
          trackRef.current.style.transform = `translateY(${-offset}px)`;
        }
      } else if (startedAt.current !== null && pauseBegan.current === null) {
        pauseBegan.current = now;
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, []);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      if (event.target instanceof HTMLInputElement) {
        return;
      }
      if (event.code === "Space") {
        event.preventDefault();
        setPaused((value) => !value);
      } else if (event.key === "ArrowUp") {
        nudgeRef.current = nudgeOffset(nudgeRef.current, -textSize);
        followRef.current = null;
      } else if (event.key === "ArrowDown") {
        nudgeRef.current = nudgeOffset(nudgeRef.current, textSize);
        followRef.current = null;
      } else if (
        event.metaKey &&
        event.shiftKey &&
        event.key.toLowerCase() === "p"
      ) {
        event.preventDefault();
        setShown((value) => !value);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, [textSize]);

  function onHeard(heard: string) {
    const match = fuzzyMatch(lines, heard);
    const mode = scrollMode(match.confidence, heard.trim().length === 0);
    if (mode === "follow") {
      followRef.current = followOffsetPx(match.index, textSize + 18);
    } else if (mode === "fixed") {
      followRef.current = null;
    }
  }

  return (
    <div className={styles.root} style={{ opacity }} data-testid="prompter">
      {shown ? (
        <div
          className={styles.script}
          data-mirror={mirror}
          data-under={underWebcam}
        >
          <div className={styles.track} ref={trackRef}>
            {lines.map((line, index) => (
              <p
                key={`${index}-${line}`}
                className={line.startsWith("[CUE]") ? styles.cue : styles.line}
                style={{
                  fontSize: line.startsWith("[CUE]")
                    ? textSize * 0.45
                    : textSize,
                }}
              >
                {line}
              </p>
            ))}
          </div>
        </div>
      ) : null}
      <form
        className={styles.controls}
        data-testid="prompter-controls"
        onSubmit={(event) => {
          event.preventDefault();
          const data = new FormData(event.currentTarget);
          onHeard(String(data.get("heard") ?? ""));
          event.currentTarget.reset();
        }}
      >
        <label>
          Speed
          <input
            type="number"
            min={10}
            max={400}
            value={speed}
            onChange={(event) => setSpeed(Number(event.target.value) || 48)}
          />
        </label>
        <label>
          Size
          <input
            type="number"
            min={16}
            max={96}
            value={textSize}
            onChange={(event) => setTextSize(Number(event.target.value) || 48)}
          />
        </label>
        <label>
          Opacity
          <input
            type="range"
            min={0.2}
            max={1}
            step={0.05}
            value={opacity}
            onChange={(event) => setOpacity(Number(event.target.value))}
          />
        </label>
        <label>
          <input
            type="checkbox"
            checked={mirror}
            onChange={(event) => setMirror(event.target.checked)}
          />
          Mirror
        </label>
        <label>
          <input
            type="checkbox"
            checked={underWebcam}
            onChange={(event) => setUnderWebcam(event.target.checked)}
          />
          Under webcam
        </label>
        <label>
          Heard
          <input name="heard" aria-label="Heard line" />
        </label>
        <button type="submit">Match</button>
        <label>
          <input
            type="checkbox"
            onChange={(event) => {
              const enabled = event.target.checked;
              void import("@tauri-apps/api/window")
                .then(({ getCurrentWindow }) =>
                  getCurrentWindow().setIgnoreCursorEvents(enabled),
                )
                .catch(() => undefined);
            }}
          />
          Click through
        </label>
        <button
          type="button"
          onClick={() => {
            void import("@tauri-apps/api/window")
              .then(({ getCurrentWindow }) => getCurrentWindow().close())
              .catch(() => undefined);
          }}
        >
          Close
        </button>
        <span>{paused ? "Paused" : "Scrolling"}</span>
      </form>
    </div>
  );
}
