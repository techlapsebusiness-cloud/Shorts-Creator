import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useEffect, useRef, useState } from "react";
import {
  COUNTDOWN_MS,
  NUDGE_PX,
  countdownLabel,
  scrollOffsetPx,
} from "./scroll";
import type { PrompterLine, PrompterSettings } from "./bridge";
import styles from "./prompter.module.css";

const LINES: PrompterLine[] = [
  { text: "This file survives a crash.", cue: false },
  { text: "CUE: point at the folder", cue: true },
  { text: "The edit lives in project.json.", cue: false },
  { text: "Show the temp file beside the save.", cue: false },
  { text: "Rename over the last complete file.", cue: false },
  { text: "Build the next short.", cue: false },
];

function isSettings(value: unknown): value is PrompterSettings {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  return Array.isArray((value as PrompterSettings).lines);
}

export function PrompterWindow() {
  const [lines, setLines] = useState(LINES);
  const [wordsPerMinute, setWordsPerMinute] = useState(140);
  const [textSize, setTextSize] = useState(48);
  const [opacity, setOpacity] = useState(1);
  const [mirror, setMirror] = useState(false);
  const [underWebcam, setUnderWebcam] = useState(true);
  const [paused, setPaused] = useState(false);
  const [hidden, setHidden] = useState(false);
  const [armed, setArmed] = useState(false);
  const [offset, setOffset] = useState(0);
  const [count, setCount] = useState<number | null>(null);
  const armedAt = useRef<number | null>(null);
  const pausedMs = useRef(0);
  const pauseStarted = useRef<number | null>(null);
  const nudge = useRef(0);
  const pace = useRef({ wordsPerMinute: 140, textSize: 48 });

  useEffect(() => {
    document.body.dataset.window = "prompter";
    return () => {
      delete document.body.dataset.window;
    };
  }, []);

  useEffect(() => {
    pace.current = { wordsPerMinute, textSize };
  }, [wordsPerMinute, textSize]);

  useEffect(() => {
    function apply(detail: PrompterSettings) {
      setLines(detail.lines);
      setWordsPerMinute(detail.wordsPerMinute);
      setTextSize(detail.textSize);
      setOpacity(detail.opacity);
      setMirror(detail.mirror);
      setUnderWebcam(detail.underWebcam);
      setPaused(detail.paused);
    }
    function onSettings(event: Event) {
      const detail = (event as CustomEvent<PrompterSettings>).detail;
      if (isSettings(detail)) {
        apply(detail);
      }
    }
    window.addEventListener("prompter-settings", onSettings);
    let stop = () => {};
    void Promise.resolve()
      .then(() =>
        listen<PrompterSettings>("prompter-settings", (event) => {
          if (isSettings(event.payload)) {
            apply(event.payload);
          }
        }),
      )
      .then((unlisten) => {
        stop = unlisten;
      })
      .catch(() => undefined);
    return () => {
      window.removeEventListener("prompter-settings", onSettings);
      stop();
    };
  }, []);

  useEffect(() => {
    function arm() {
      armedAt.current = performance.now();
      pausedMs.current = 0;
      pauseStarted.current = null;
      nudge.current = 0;
      setOffset(0);
      setCount(3);
      setPaused(false);
      setHidden(false);
      setArmed(true);
    }
    window.addEventListener("prompter-arm", arm);
    let stop = () => {};
    void Promise.resolve()
      .then(() => listen("prompter-arm", () => arm()))
      .then((unlisten) => {
        stop = unlisten;
      })
      .catch(() => undefined);
    return () => {
      window.removeEventListener("prompter-arm", arm);
      stop();
    };
  }, []);

  useEffect(() => {
    if (!armed) {
      return;
    }
    if (paused) {
      pauseStarted.current ??= performance.now();
      return;
    }
    if (pauseStarted.current !== null) {
      pausedMs.current += performance.now() - pauseStarted.current;
      pauseStarted.current = null;
    }
    let frame = 0;
    const tick = (now: number) => {
      const started = armedAt.current;
      if (started !== null) {
        const frozen = pauseStarted.current ?? now;
        setCount(countdownLabel(frozen - started));
        setOffset(
          scrollOffsetPx({
            nowMs: frozen,
            startMs: started + COUNTDOWN_MS,
            pausedMs: pausedMs.current,
            wordsPerMinute: pace.current.wordsPerMinute,
            textSize: pace.current.textSize,
            nudgePx: nudge.current,
          }),
        );
      }
      frame = requestAnimationFrame(tick);
    };
    frame = requestAnimationFrame(tick);
    return () => cancelAnimationFrame(frame);
  }, [armed, paused]);

  useEffect(() => {
    function onKey(event: KeyboardEvent) {
      const target = event.target;
      if (
        target instanceof HTMLInputElement ||
        target instanceof HTMLTextAreaElement
      ) {
        return;
      }
      if (event.code === "Space") {
        event.preventDefault();
        setPaused((value) => !value);
      } else if (event.code === "ArrowUp" || event.code === "ArrowDown") {
        event.preventDefault();
        nudge.current += event.code === "ArrowUp" ? -NUDGE_PX : NUDGE_PX;
      } else if (
        event.shiftKey &&
        (event.metaKey || event.ctrlKey) &&
        event.code === "KeyP"
      ) {
        event.preventDefault();
        setHidden((value) => !value);
      }
    }
    window.addEventListener("keydown", onKey);
    return () => window.removeEventListener("keydown", onKey);
  }, []);

  function cursor(ignore: boolean) {
    void invoke("prompter_click_through", { ignore }).catch(() => undefined);
  }

  let status = "Ready";
  if (hidden) {
    status = "Prompter hidden";
  } else if (count !== null) {
    status = String(count);
  } else if (paused) {
    status = "Paused";
  } else if (armed) {
    status = "Scrolling";
  }

  const shown = lines.filter((line) => line.text.trim() !== "");

  return (
    <section className={styles.shell} aria-label="Prompter">
      <div
        className={`${styles.scroll} ${underWebcam ? styles.under : ""}`}
        style={{
          opacity,
          transform: mirror ? "scaleX(-1)" : undefined,
        }}
      >
        <p className={styles.banner}>
          <span className={styles.dot} />
          PROMPTER · NOT RECORDED
        </p>
        {hidden
          ? null
          : shown.map((line) => (
              <p
                key={line.text}
                className={
                  line.cue ? `${styles.line} ${styles.cue}` : styles.line
                }
                data-cue={line.cue ? "true" : "false"}
                style={{
                  fontSize: textSize,
                  transform: `translateY(${-offset}px)`,
                }}
              >
                {line.text}
              </p>
            ))}
      </div>
      <div
        className={styles.controls}
        onPointerEnter={() => cursor(false)}
        onPointerLeave={() => cursor(true)}
      >
        <button type="button" onClick={() => setPaused((value) => !value)}>
          {paused ? "Resume" : "Pause"}
        </button>
        <p className={styles.status} role="status">
          {status}
        </p>
      </div>
    </section>
  );
}
