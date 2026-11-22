import { invoke } from "@tauri-apps/api/core";
import type { Project } from "../bindings/Project";
import { DailyCard } from "../daily/DailyCard";
import { useEffect, useRef, useState } from "react";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Field, SelectField } from "../components/Field";
import { Pill } from "../components/Pill";
import { Toggle } from "../components/Toggle";
import styles from "./screens.module.css";

const CHIPS = [
  "Hook in 3 s",
  "Screen cues",
  "Fact-check notes",
  "CTA",
] as const;
const BEATS = [
  ["HOOK", "0:00", "This file survives a crash."],
  ["CONCEPT", "0:03", "The edit lives in project.json."],
  ["DEMO", "0:08", "Show the temp file beside the save."],
  ["FIX", "0:15", "Rename over the last complete file."],
  ["CTA", "0:22", "Build the next short."],
] as const;

type LayoutId = "screenCam" | "camOnly" | "screenOnly";
type Phase = "idle" | "countdown" | "recording" | "paused";

type CaptureDevice = { id: string; label: string; kind: string };
type DeviceList = {
  screens: CaptureDevice[];
  cameras: CaptureDevice[];
  microphones: CaptureDevice[];
  backend: string;
};
type CaptureWindow = { id: number; title: string };
type CaptureRegion = { x: number; y: number; width: number; height: number };
type DiskEstimate = {
  freeBytes: number;
  bytesPerMinute: number;
  minutesRemaining: number;
};
type ScriptBeat = {
  kind: string;
  start: number;
  text: string;
  cues: string[];
};
type ScriptDraft = {
  raw: string;
  factCheck: string[];
  beats: ScriptBeat[];
};
type TakeResult = {
  screen: string | null;
  cam: string | null;
  mic: string | null;
  cursor: string;
  outDir?: string;
  sync: {
    screenOffsetMs: number;
    camOffsetMs: number;
    micOffsetMs: number;
  };
};

const ALTERNATES = [
  "Wait — this file survives a crash.",
  "The project file is the edit.",
  "Watch the temp file land beside the save.",
  "The rename replaces the last good file.",
  "Make the next short.",
];

const LAYOUTS: { id: LayoutId; label: string }[] = [
  { id: "screenCam", label: "Screen + cam" },
  { id: "camOnly", label: "Cam only" },
  { id: "screenOnly", label: "Screen only" },
];

type TakeSummary = {
  id: string;
  dir: string;
  createdMs: number;
  durationSecs: number;
  screen: boolean;
  cam: boolean;
  mic: boolean;
};

type OpenedTake = {
  project: Project;
  dir: string;
};

type RecordScreenProps = {
  actionToken?: number;
  /** Test override. Production uses a 3 s countdown. */
  countdownSecs?: number;
  /** Test override for lavfi auto-stop hold. */
  lavfiHoldMs?: number;
  onOpened?: (opened: OpenedTake) => void;
};

export function RecordScreen({
  actionToken = 0,
  countdownSecs = 3,
  lavfiHoldMs = 3400,
  onOpened,
}: RecordScreenProps) {
  const [model, setModel] = useState<"Claude" | "ChatGPT">("Claude");
  const [topic, setTopic] = useState("crash-safe saves");
  const [language, setLanguage] = useState("Hinglish");
  const [length, setLength] = useState("30 s");
  const [chips, setChips] = useState<string[]>(["Hook in 3 s", "CTA"]);
  const [beats, setBeats] = useState<string[]>(BEATS.map((beat) => beat[2]));
  const [draft, setDraft] = useState<ScriptDraft | null>(null);
  const [followsVoice, setFollowsVoice] = useState(true);
  const [mirror, setMirror] = useState(false);
  const [underWebcam, setUnderWebcam] = useState(true);
  const [cues, setCues] = useState(true);
  const [layout, setLayout] = useState<LayoutId>("screenCam");
  const [devices, setDevices] = useState<DeviceList | null>(null);
  const [screenId, setScreenId] = useState("");
  const [camId, setCamId] = useState("");
  const [micId, setMicId] = useState("");
  const [backend, setBackend] = useState("lavfi");
  const [windows, setWindows] = useState<CaptureWindow[]>([]);
  const [windowId, setWindowId] = useState<number | null>(null);
  const [regionText, setRegionText] = useState("");
  const [iconSupport, setIconSupport] = useState(false);
  const [iconsHidden, setIconsHidden] = useState(false);
  const [micLevel, setMicLevel] = useState(0);
  const [disk, setDisk] = useState<DiskEstimate | null>(null);
  const [takeBytes, setTakeBytes] = useState<number | null>(null);
  const [phase, setPhase] = useState<Phase>("idle");
  const [countdown, setCountdown] = useState(0);
  const [status, setStatus] = useState("Ready");
  const [lastTake, setLastTake] = useState<TakeResult | null>(null);
  const [takes, setTakes] = useState<TakeSummary[]>([]);
  const [error, setError] = useState("");
  const [camError, setCamError] = useState("");
  const previewRef = useRef<HTMLVideoElement>(null);
  const finishRef = useRef<() => Promise<void>>(async () => undefined);

  useEffect(() => {
    let cancelled = false;
    invoke<DeviceList>("list_capture_devices")
      .then((list) => {
        if (cancelled) {
          return;
        }
        setDevices(list);
        setScreenId(
          pickDevice(list.screens, (device) =>
            device.label.toLowerCase().includes("capture screen"),
          ),
        );
        setCamId(
          pickDevice(
            list.cameras,
            (device) => !device.label.toLowerCase().includes("iphone"),
          ),
        );
        setMicId(
          pickDevice(
            list.microphones,
            (device) => !device.label.toLowerCase().includes("iphone"),
          ),
        );
        const hasReal =
          list.screens.length > 0 ||
          list.cameras.length > 0 ||
          list.microphones.length > 0;
        if (hasReal && list.backend === "avfoundation") {
          setBackend("avfoundation");
        } else {
          setBackend("lavfi");
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setError(String(err));
        }
      });
    invoke<DiskEstimate>("disk_estimate", { path: "." })
      .then((value) => {
        if (!cancelled) {
          setDisk(value);
        }
      })
      .catch(() => undefined);
    invoke<CaptureWindow[]>("list_capture_windows")
      .then((list) => {
        if (!cancelled) {
          setWindows(list);
        }
      })
      .catch(() => undefined);
    invoke<TakeSummary[]>("list_takes")
      .then((list) => {
        if (!cancelled) {
          setTakes(list);
        }
      })
      .catch(() => undefined);
    invoke<{ supported: boolean }>("desktop_icon_support")
      .then((value) => {
        if (!cancelled) {
          setIconSupport(value.supported);
        }
      })
      .catch(() => undefined);
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const tauri = "__TAURI_INTERNALS__" in window;
    if (!tauri || !navigator.mediaDevices?.getUserMedia) {
      return;
    }
    let stream: MediaStream | null = null;
    let cancelled = false;
    navigator.mediaDevices
      .getUserMedia({ video: { facingMode: "user" }, audio: false })
      .then((next) => {
        if (cancelled || next.getVideoTracks().length === 0) {
          next.getTracks().forEach((track) => track.stop());
          return;
        }
        stream = next;
        const preview = previewRef.current;
        if (preview) {
          preview.srcObject = next;
          preview.dataset.live = "true";
        }
      })
      .catch((err: unknown) => {
        if (!cancelled) {
          setCamError(err instanceof Error ? err.message : String(err));
        }
      });
    return () => {
      cancelled = true;
      stream?.getTracks().forEach((track) => track.stop());
    };
  }, []);

  useEffect(() => {
    if (phase !== "recording") {
      return;
    }
    function onPointer(event: PointerEvent) {
      void invoke("log_overlay_event", {
        kind: "click",
        label: String(event.button),
      }).catch(() => undefined);
    }
    function onKey(event: KeyboardEvent) {
      if (event.metaKey || event.ctrlKey || event.altKey) {
        return;
      }
      void invoke("log_overlay_event", { kind: "key", label: event.key }).catch(
        () => undefined,
      );
    }
    window.addEventListener("pointerdown", onPointer);
    window.addEventListener("keydown", onKey);
    return () => {
      window.removeEventListener("pointerdown", onPointer);
      window.removeEventListener("keydown", onKey);
    };
  }, [phase]);

  useEffect(() => {
    if (phase !== "recording" || !("__TAURI_INTERNALS__" in window)) {
      return;
    }
    let stream: MediaStream | null = null;
    let context: AudioContext | null = null;
    let frame = 0;
    let alive = true;
    navigator.mediaDevices
      ?.getUserMedia({ audio: true, video: false })
      .then((next) => {
        if (!alive || next.getAudioTracks().length === 0) {
          next.getTracks().forEach((track) => track.stop());
          return;
        }
        stream = next;
        context = new AudioContext();
        const source = context.createMediaStreamSource(next);
        const analyser = context.createAnalyser();
        analyser.fftSize = 256;
        source.connect(analyser);
        const data = new Uint8Array(analyser.fftSize);
        const tick = () => {
          analyser.getByteTimeDomainData(data);
          let sum = 0;
          for (const value of data) {
            const sample = (value - 128) / 128;
            sum += sample * sample;
          }
          setMicLevel(Math.min(1, Math.sqrt(sum / data.length) * 4));
          frame = window.requestAnimationFrame(tick);
        };
        tick();
      })
      .catch(() => undefined);
    return () => {
      alive = false;
      window.cancelAnimationFrame(frame);
      stream?.getTracks().forEach((track) => track.stop());
      void context?.close();
    };
  }, [phase]);

  useEffect(() => {
    if (phase !== "recording" && phase !== "paused") {
      setTakeBytes(null);
      return;
    }
    let cancel = false;
    const poll = async () => {
      while (!cancel) {
        const status = await invoke<{
          bytes: number;
          message: string | null;
        }>("recording_status").catch(() => null);
        if (cancel) {
          return;
        }
        if (status) {
          setTakeBytes(status.bytes);
          if (status.message) {
            await finishRef.current();
            return;
          }
        }
        await wait(1000);
        if (cancel) {
          return;
        }
      }
    };
    void poll();
    return () => {
      cancel = true;
    };
  }, [phase]);

  useEffect(() => {
    if (actionToken > 0) {
      void beginRecording();
    }
  }, [actionToken]);

  async function generateScript() {
    setError("");
    try {
      const view = await invoke<{ draft: ScriptDraft }>("generate_script", {
        topic,
        language,
        lengthSecs: lengthSeconds(length),
        wpm: 140,
      });
      setDraft(view.draft);
      setBeats(view.draft.beats.map((beat) => beat.text));
      await persistScript(
        view.draft,
        view.draft.beats.map((beat) => beat.text),
      );
    } catch (err) {
      setError(String(err));
    }
  }

  async function regenerateLine(index: number) {
    const current = draft ?? draftFromBeats(beats, topic, language);
    const kind = BEATS[index]?.[0].toLowerCase() ?? "hook";
    try {
      const view = await invoke<{ draft: ScriptDraft }>("regenerate_beat", {
        draft: {
          ...current,
          beats: current.beats.map((beat, beatIndex) =>
            beatIndex === index
              ? { ...beat, text: beats[beatIndex] ?? beat.text }
              : beat,
          ),
        },
        kind,
        text: ALTERNATES[index] ?? beats[index],
        cues: current.beats[index]?.cues ?? [],
        wpm: 140,
      });
      setDraft(view.draft);
      setBeats(view.draft.beats.map((beat) => beat.text));
      await persistScript(
        view.draft,
        view.draft.beats.map((beat) => beat.text),
      );
    } catch (err) {
      setError(String(err));
    }
  }

  async function persistScript(next: ScriptDraft, lines: string[]) {
    const saved = {
      ...next,
      beats: next.beats.map((beat, index) => ({
        ...beat,
        text: lines[index] ?? beat.text,
      })),
    };
    await invoke("save_script", { topic, language, draft: saved }).catch(
      () => undefined,
    );
  }

  async function beginRecording() {
    if (phase !== "idle") {
      return;
    }
    setError("");
    setLastTake(null);
    setPhase("countdown");
    void invoke("prompter_set_running", { running: true }).catch(
      () => undefined,
    );
    for (let value = countdownSecs; value >= 1; value -= 1) {
      setCountdown(value);
      setStatus(`Countdown ${value}`);
      await wait(1000);
    }
    setCountdown(0);
    try {
      const outDir = await invoke<string>("prepare_take");
      await invoke("start_recording", {
        args: {
          outDir,
          layout,
          backend,
          screenId: screenId || null,
          camId: camId || null,
          micId: micId || null,
          countdownSecs: 0,
          fixedDurationSecs: backend === "lavfi" ? 3 : null,
          windowId,
          region: parseRegion(regionText),
        },
      });
      setPhase("recording");
      setStatus(
        backend === "lavfi" ? "Recording fixture take (3 s)…" : "Recording…",
      );
      if (backend === "lavfi") {
        await wait(lavfiHoldMs);
        await finishRecording();
      }
    } catch (err) {
      void invoke("prompter_set_running", { running: false }).catch(
        () => undefined,
      );
      setPhase("idle");
      setStatus("Ready");
      setError(String(err));
    }
  }

  async function finishRecording() {
    try {
      const take = await invoke<TakeResult>("stop_recording");
      void invoke("prompter_set_running", { running: false }).catch(
        () => undefined,
      );
      setLastTake(take);
      setPhase("idle");
      setStatus(take.outDir ? `Take saved · ${take.outDir}` : "Take saved");
      const listed = await invoke<TakeSummary[]>("list_takes").catch(() => []);
      setTakes(listed);
    } catch (err) {
      void invoke("prompter_set_running", { running: false }).catch(
        () => undefined,
      );
      setPhase("idle");
      setStatus("Ready");
      setError(String(err));
    }
  }

  finishRef.current = finishRecording;

  async function togglePause() {
    try {
      if (phase === "recording") {
        await invoke("pause_recording");
        void invoke("prompter_set_running", { running: false }).catch(
          () => undefined,
        );
        setPhase("paused");
        setStatus("Paused");
      } else if (phase === "paused") {
        await invoke("resume_recording");
        void invoke("prompter_set_running", { running: true }).catch(
          () => undefined,
        );
        setPhase("recording");
        setStatus("Recording…");
      }
    } catch (err) {
      setError(String(err));
    }
  }

  const busy =
    phase === "countdown" || phase === "recording" || phase === "paused";

  return (
    <div className={`${styles.page} ${styles.columns}`}>
      <DailyCard />
      <Card title="Script">
        <div className={styles.row}>
          <Pill
            selected={model === "Claude"}
            onClick={() => setModel("Claude")}
          >
            Claude
          </Pill>
          <Pill
            selected={model === "ChatGPT"}
            onClick={() => setModel("ChatGPT")}
          >
            ChatGPT
          </Pill>
        </div>
        <Field label="Topic" value={topic} onChange={setTopic} />
        <SelectField
          label="Language"
          value={language}
          options={["Hinglish", "English", "Hindi"]}
          onChange={setLanguage}
        />
        <SelectField
          label="Length"
          value={length}
          options={["15 s", "30 s", "45 s", "60 s"]}
          onChange={setLength}
        />
        <div className={styles.row}>
          {CHIPS.map((chip) => (
            <Pill
              key={chip}
              selected={chips.includes(chip)}
              onClick={() =>
                setChips((current) =>
                  current.includes(chip)
                    ? current.filter((item) => item !== chip)
                    : [...current, chip],
                )
              }
            >
              {chip}
            </Pill>
          ))}
        </div>
        <Button tone="primary" onClick={() => void generateScript()}>
          Generate
        </Button>
        <p className={styles.meta} data-testid="script-estimate">
          {scriptWords(beats)} words · {scriptSeconds(beats).toFixed(1)} s at
          140 wpm
        </p>
        {BEATS.map((beat, index) => (
          <label key={beat[0]} className={styles.beat}>
            <span className={styles.kind}>{beat[0]}</span>
            <span className={styles.meta}>{beat[1]}</span>
            <input
              className={styles.line}
              value={beats[index]}
              aria-label={`${beat[0]} beat`}
              onChange={(event) =>
                setBeats((current) =>
                  current.map((line, lineIndex) =>
                    lineIndex === index ? event.target.value : line,
                  ),
                )
              }
              onBlur={() => {
                if (draft) {
                  void persistScript(draft, beats);
                }
              }}
            />
            <button
              type="button"
              className={styles.tool}
              onClick={() => void regenerateLine(index)}
            >
              Regenerate
            </button>
          </label>
        ))}
      </Card>
      <div className={styles.stack}>
        <div className={styles.stage}>
          <div className={styles.pair}>
            <p className={styles.kicker}>What you see</p>
            <div className={styles.prompter}>
              <p className={styles.banner}>
                <span className={styles.dot} />
                PROMPTER · NOT RECORDED
              </p>
              <button
                type="button"
                className={styles.tool}
                onClick={() => {
                  void invoke("open_prompter").catch(() => undefined);
                }}
              >
                Open prompter
              </button>
              <p className={styles.promptLine}>{beats[0]}</p>
            </div>
          </div>
          <div className={styles.pair}>
            <p className={styles.kicker}>What gets recorded</p>
            <div className={styles.recorded}>
              <video
                ref={previewRef}
                className={styles.preview}
                aria-label="Camera preview"
                autoPlay
                muted
                playsInline
              />
              {micLevel > 0.02 ? (
                <span
                  className={styles.meter}
                  style={{ height: `${Math.round(micLevel * 100)}%` }}
                  aria-label="Microphone level"
                />
              ) : null}
              <p className={styles.banner}>
                {countdown > 0 ? countdown : "9:16"}
              </p>
              <p className={styles.meta} data-testid="record-status">
                {camError || status}
              </p>
            </div>
          </div>
        </div>
        <SelectField
          label="Backend"
          value={backend}
          options={
            devices?.backend === "avfoundation"
              ? ["lavfi", "avfoundation"]
              : ["lavfi", "linux"]
          }
          onChange={setBackend}
        />
        <SelectField
          label="Screen"
          value={deviceLabel(
            devices?.screens ?? [],
            screenId,
            "No screen found",
          )}
          options={deviceLabels(devices?.screens ?? [], "No screen found")}
          onChange={(value) =>
            setScreenId(deviceIdFromLabel(devices?.screens ?? [], value))
          }
        />
        <SelectField
          label="Camera"
          value={deviceLabel(devices?.cameras ?? [], camId, "No camera found")}
          options={deviceLabels(devices?.cameras ?? [], "No camera found")}
          onChange={(value) =>
            setCamId(deviceIdFromLabel(devices?.cameras ?? [], value))
          }
        />
        <SelectField
          label="Microphone"
          value={deviceLabel(
            devices?.microphones ?? [],
            micId,
            "No microphone found",
          )}
          options={deviceLabels(
            devices?.microphones ?? [],
            "No microphone found",
          )}
          onChange={(value) =>
            setMicId(deviceIdFromLabel(devices?.microphones ?? [], value))
          }
        />
        <div className={styles.row}>
          {LAYOUTS.map((item) => (
            <Pill
              key={item.id}
              selected={layout === item.id}
              onClick={() => setLayout(item.id)}
            >
              {item.label}
            </Pill>
          ))}
          <Button
            tone="primary"
            disabled={busy}
            onClick={() => void beginRecording()}
          >
            Start recording
          </Button>
          <Button
            tone="danger"
            disabled={phase !== "recording" && phase !== "paused"}
            onClick={() => void finishRecording()}
          >
            Stop recording
          </Button>
          <Button
            disabled={phase !== "recording" && phase !== "paused"}
            onClick={() => void togglePause()}
          >
            {phase === "paused" ? "Resume" : "Pause"}
          </Button>
        </div>
        {error ? (
          <p className={styles.meta} role="alert">
            {error}
          </p>
        ) : null}
        {lastTake ? (
          <p className={styles.meta} data-testid="last-take">
            Saved {lastTake.screen ?? "—"} · {lastTake.cam ?? "—"} ·{" "}
            {lastTake.mic ?? "—"} · {lastTake.cursor}
            {lastTake.outDir ? ` · ${lastTake.outDir}` : ""}
          </p>
        ) : null}
        {disk ? (
          <p className={styles.meta}>
            Disk ~{Math.floor(disk.minutesRemaining)} min at{" "}
            {Math.round(disk.bytesPerMinute / 1_000_000)} MB/min
            {takeBytes !== null ? ` · this take ${formatBytes(takeBytes)}` : ""}
          </p>
        ) : null}
        <Card title="Recordings">
          {takes.length === 0 ? (
            <p className={styles.meta}>No recordings yet.</p>
          ) : (
            takes.map((take) => (
              <div key={take.id} className={styles.row}>
                <p className={styles.meta}>
                  {take.id} · {take.durationSecs.toFixed(1)}s
                  {take.screen ? " · screen" : ""}
                  {take.cam ? " · cam" : ""}
                  {take.mic ? " · mic" : ""}
                </p>
                <Button
                  onClick={() => {
                    void invoke<OpenedTake>("open_take", { takeDir: take.dir })
                      .then((opened) => onOpened?.(opened))
                      .catch((err: unknown) => setError(String(err)));
                  }}
                >
                  Open in editor
                </Button>
              </div>
            ))
          )}
        </Card>
      </div>
      <Card title="Prompter">
        <div className={styles.row}>
          <Pill selected={followsVoice} onClick={() => setFollowsVoice(true)}>
            Follows voice
          </Pill>
          <Pill selected={!followsVoice} onClick={() => setFollowsVoice(false)}>
            Fixed speed
          </Pill>
        </div>
        <Field label="Speed" value="140 wpm" onChange={() => undefined} />
        <Field label="Text size" value="48" onChange={() => undefined} />
        <Toggle label="Mirror" checked={mirror} onChange={setMirror} />
        <Toggle
          label="Position under webcam"
          checked={underWebcam}
          onChange={setUnderWebcam}
        />
        <Toggle label="Show screen cues" checked={cues} onChange={setCues} />
        <Toggle label="Hide from recording" checked disabled />
        {iconSupport ? (
          <Toggle
            label="Hide desktop icons"
            checked={iconsHidden}
            onChange={(checked) => {
              setIconsHidden(checked);
              void invoke("set_desktop_icons", { visible: !checked }).catch(
                (err: unknown) => setError(String(err)),
              );
            }}
          />
        ) : null}
        {iconSupport ? (
          <Field
            label="Region x, y, width, height"
            value={regionText}
            onChange={setRegionText}
          />
        ) : null}
        {(windows?.length ?? 0) > 0 ? (
          <SelectField
            label="Window"
            value={windowId === null ? "display" : String(windowId)}
            options={[
              "display",
              ...windows.map((item) => `${item.id} · ${item.title}`),
            ]}
            onChange={(value) => {
              if (value === "display") {
                setWindowId(null);
                return;
              }
              const id = Number(value.split(" · ")[0]);
              setWindowId(Number.isFinite(id) ? id : null);
            }}
          />
        ) : null}
      </Card>
    </div>
  );
}

function scriptWords(lines: string[]) {
  return lines
    .join(" ")
    .split(/\s+/)
    .filter((word) => word.length > 0).length;
}

function scriptSeconds(lines: string[]) {
  return (scriptWords(lines) / 140) * 60;
}

function lengthSeconds(value: string) {
  const parsed = Number(value.replace(/[^0-9]/g, ""));
  return Number.isFinite(parsed) && parsed > 0 ? parsed : 30;
}

function draftFromBeats(
  lines: string[],
  topic: string,
  language: string,
): ScriptDraft {
  return {
    raw: `${language} draft about ${topic}. ${lines.join(" ")}`,
    factCheck: [],
    beats: BEATS.map((beat, index) => ({
      kind: beat[0].toLowerCase(),
      start: index === 0 ? 0 : index * 5,
      text: lines[index] ?? "",
      cues: [],
    })),
  };
}

function parseRegion(value: string): CaptureRegion | null {
  const parts = value.split(",").map((part) => Number(part.trim()));
  if (
    parts.length !== 4 ||
    parts.some((part) => !Number.isFinite(part) || part < 0)
  ) {
    return null;
  }
  if (parts[2] === 0 || parts[3] === 0) {
    return null;
  }
  return { x: parts[0], y: parts[1], width: parts[2], height: parts[3] };
}

function pickDevice(
  devices: CaptureDevice[],
  prefer: (device: CaptureDevice) => boolean,
) {
  return (devices.find(prefer) ?? devices[0])?.id ?? "";
}

function deviceLabels(devices: CaptureDevice[], empty: string) {
  if (devices.length === 0) {
    return [empty];
  }
  return devices.map((device) => device.label || device.id);
}

function deviceLabel(devices: CaptureDevice[], id: string, empty: string) {
  return devices.find((device) => device.id === id)?.label || empty;
}

function deviceIdFromLabel(devices: CaptureDevice[], label: string) {
  return (
    devices.find((device) => (device.label || device.id) === label)?.id ?? ""
  );
}

function wait(ms: number) {
  return new Promise((resolve) => {
    window.setTimeout(resolve, ms);
  });
}

function formatBytes(bytes: number) {
  if (bytes >= 1_000_000_000) {
    return `${(bytes / 1_000_000_000).toFixed(1)} GB`;
  }
  if (bytes >= 1_000_000) {
    return `${Math.round(bytes / 1_000_000)} MB`;
  }
  return `${Math.round(bytes / 1_000)} KB`;
}
