import { invoke } from "@tauri-apps/api/core";
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
type DiskEstimate = {
  freeBytes: number;
  bytesPerMinute: number;
  minutesRemaining: number;
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

const LAYOUTS: { id: LayoutId; label: string }[] = [
  { id: "screenCam", label: "Screen + cam" },
  { id: "camOnly", label: "Cam only" },
  { id: "screenOnly", label: "Screen only" },
];

type RecordScreenProps = {
  actionToken?: number;
  /** Test override. Production uses a 3 s countdown. */
  countdownSecs?: number;
  /** Test override for lavfi auto-stop hold. */
  lavfiHoldMs?: number;
};

export function RecordScreen({
  actionToken = 0,
  countdownSecs = 3,
  lavfiHoldMs = 3400,
}: RecordScreenProps) {
  const [model, setModel] = useState<"Claude" | "ChatGPT">("Claude");
  const [topic, setTopic] = useState("crash-safe saves");
  const [language, setLanguage] = useState("Hinglish");
  const [length, setLength] = useState("30 s");
  const [chips, setChips] = useState<string[]>(["Hook in 3 s", "CTA"]);
  const [beats, setBeats] = useState<string[]>(BEATS.map((beat) => beat[2]));
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
  const [disk, setDisk] = useState<DiskEstimate | null>(null);
  const [phase, setPhase] = useState<Phase>("idle");
  const [countdown, setCountdown] = useState(0);
  const [status, setStatus] = useState("Ready");
  const [lastTake, setLastTake] = useState<TakeResult | null>(null);
  const [error, setError] = useState("");
  const [camError, setCamError] = useState("");
  const previewRef = useRef<HTMLVideoElement>(null);

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
    if (actionToken > 0) {
      void beginRecording();
    }
  }, [actionToken]);

  async function beginRecording() {
    if (phase !== "idle") {
      return;
    }
    setError("");
    setLastTake(null);
    setPhase("countdown");
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
      setPhase("idle");
      setStatus("Ready");
      setError(String(err));
    }
  }

  async function finishRecording() {
    try {
      const take = await invoke<TakeResult>("stop_recording");
      setLastTake(take);
      setPhase("idle");
      setStatus(take.outDir ? `Take saved · ${take.outDir}` : "Take saved");
    } catch (err) {
      setPhase("idle");
      setStatus("Ready");
      setError(String(err));
    }
  }

  async function togglePause() {
    try {
      if (phase === "recording") {
        await invoke("pause_recording");
        setPhase("paused");
        setStatus("Paused");
      } else if (phase === "paused") {
        await invoke("resume_recording");
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
        <Button tone="primary">Generate</Button>
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
            />
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
              <p className={styles.banner}>
                {countdown > 0 ? countdown : "9:16"}
              </p>
              <p className={styles.meta} data-testid="record-status">
                {camError || status}
              </p>
            </div>
          </div>
        </div>
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
          </p>
        ) : null}
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
          value={deviceOption(devices?.screens ?? [], screenId)}
          options={
            devices?.screens.length
              ? devices.screens.map(deviceOptionLabel)
              : ["lavfi"]
          }
          onChange={(value) => setScreenId(deviceId(value))}
        />
        <SelectField
          label="Camera"
          value={deviceOption(devices?.cameras ?? [], camId)}
          options={
            devices?.cameras.length
              ? devices.cameras.map(deviceOptionLabel)
              : ["lavfi"]
          }
          onChange={(value) => setCamId(deviceId(value))}
        />
        <SelectField
          label="Microphone"
          value={deviceOption(devices?.microphones ?? [], micId)}
          options={
            devices?.microphones.length
              ? devices.microphones.map(deviceOptionLabel)
              : ["lavfi"]
          }
          onChange={(value) => setMicId(deviceId(value))}
        />
      </Card>
    </div>
  );
}

function pickDevice(
  devices: CaptureDevice[],
  prefer: (device: CaptureDevice) => boolean,
) {
  return (devices.find(prefer) ?? devices[0])?.id ?? "";
}

function deviceOptionLabel(device: CaptureDevice) {
  return `${device.id} · ${device.label}`;
}

function deviceOption(devices: CaptureDevice[], id: string) {
  const device = devices.find((item) => item.id === id);
  return device ? deviceOptionLabel(device) : id || "lavfi";
}

function deviceId(value: string) {
  return value.split(" · ")[0] ?? value;
}

function wait(ms: number) {
  return new Promise((resolve) => {
    window.setTimeout(resolve, ms);
  });
}
