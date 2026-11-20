import { useEffect, useState } from "react";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Field, SelectField } from "../components/Field";
import { Pill } from "../components/Pill";
import { Toggle } from "../components/Toggle";
import {
  WORDS_PER_MINUTE,
  estimateSeconds,
  spokenText,
  wordCount,
} from "../script/estimate";
import { publishPrompter, type PrompterLine } from "../prompter/bridge";
import {
  AUDIO_BITRATE_KBPS,
  VIDEO_BITRATE_KBPS,
  estimateMegabytes,
} from "../record/estimate";
import styles from "./screens.module.css";

const CHIPS = [
  "Hook in 3 s",
  "Screen cues",
  "Fact-check notes",
  "CTA",
] as const;

export type ScriptBeat = {
  kind: string;
  start: number;
  text: string;
  cues: string[];
};

export type ScriptBrief = {
  provider: string;
  model: string;
  topic: string;
  language: string;
  lengthSeconds: number;
  wordsPerMinute: number;
  hookIn3s: boolean;
  screenCues: boolean;
  factCheck: boolean;
  cta: boolean;
};

export type GeneratedScript = {
  rawDraft: string;
  beats: ScriptBeat[];
};

const INITIAL: ScriptBeat[] = [
  { kind: "hook", start: 0, text: "This file survives a crash.", cues: [] },
  {
    kind: "concept",
    start: 3,
    text: "The edit lives in project.json.",
    cues: [],
  },
  {
    kind: "demo",
    start: 8,
    text: "Show the temp file beside the save.",
    cues: [],
  },
  {
    kind: "fix",
    start: 15,
    text: "Rename over the last complete file.",
    cues: [],
  },
  { kind: "cta", start: 22, text: "Build the next short.", cues: [] },
];

const LABELS: Record<string, string> = {
  hook: "HOOK",
  concept: "CONCEPT",
  demo: "DEMO",
  fix: "FIX",
  cta: "CTA",
};

type RecordScreenProps = {
  claudeModel?: string;
  openaiModel?: string;
  wordsPerMinute?: number;
  captureNote?: string;
  onGenerate?: (brief: ScriptBrief) => Promise<GeneratedScript>;
  onRegenerate?: (
    brief: ScriptBrief,
    index: number,
    beats: ScriptBeat[],
  ) => Promise<ScriptBeat>;
};

function leadingNumber(value: string, fallback: number): number {
  const parsed = Number.parseFloat(value);
  return Number.isFinite(parsed) && parsed > 0 ? parsed : fallback;
}

function opacityOf(value: string): number {
  const parsed = Number.parseFloat(value);
  if (!Number.isFinite(parsed) || parsed <= 0) {
    return 1;
  }
  return parsed > 1 ? Math.min(parsed, 100) / 100 : parsed;
}

function prompterLines(beats: ScriptBeat[], showCues: boolean): PrompterLine[] {
  const lines: PrompterLine[] = [];
  for (const beat of beats) {
    if (beat.text.trim() !== "") {
      lines.push({ text: beat.text, cue: false });
    }
    if (showCues) {
      for (const cue of beat.cues) {
        if (cue.startsWith("CUE:")) {
          lines.push({ text: cue, cue: true });
        }
      }
    }
  }
  return lines;
}

function clock(start: number): string {
  const total = Math.max(0, Math.floor(start));
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${seconds.toString().padStart(2, "0")}`;
}

function lengthSeconds(label: string): number {
  const value = Number.parseInt(label, 10);
  return Number.isFinite(value) ? value : 30;
}

function noticeFrom(error: unknown): string {
  if (typeof error === "string" && error.trim() !== "") {
    return error;
  }
  if (error instanceof Error && error.message.trim() !== "") {
    return error.message;
  }
  return "Generate failed";
}

export function RecordScreen({
  claudeModel = "",
  openaiModel = "",
  wordsPerMinute = WORDS_PER_MINUTE,
  captureNote = "",
  onGenerate,
  onRegenerate,
}: RecordScreenProps) {
  const [model, setModel] = useState<"Claude" | "ChatGPT">("Claude");
  const [topic, setTopic] = useState("crash-safe saves");
  const [language, setLanguage] = useState("Hinglish");
  const [length, setLength] = useState("30 s");
  const [chips, setChips] = useState<string[]>(["Hook in 3 s", "CTA"]);
  const [beats, setBeats] = useState<ScriptBeat[]>(INITIAL);
  const [rawDraft, setRawDraft] = useState("");
  const [notice, setNotice] = useState("");
  const [followsVoice, setFollowsVoice] = useState(true);
  const [mirror, setMirror] = useState(false);
  const [underWebcam, setUnderWebcam] = useState(true);
  const [cues, setCues] = useState(true);
  const [preset, setPreset] = useState("Screen + cam");
  const [speed, setSpeed] = useState(`${wordsPerMinute} wpm`);
  const [textSize, setTextSize] = useState("48");
  const [opacity, setOpacity] = useState("100%");
  const [paused, setPaused] = useState(false);
  const [display, setDisplay] = useState("Fixture display");
  const [camera, setCamera] = useState("Fixture camera");
  const [microphone, setMicrophone] = useState("Fixture microphone");
  const [highlightClicks, setHighlightClicks] = useState(true);
  const [showKeys, setShowKeys] = useState(false);
  const disk = estimateMegabytes(30, VIDEO_BITRATE_KBPS, AUDIO_BITRATE_KBPS, 2);

  const pace = leadingNumber(speed, wordsPerMinute);
  const size = leadingNumber(textSize, 48);

  useEffect(() => {
    void publishPrompter({
      lines: prompterLines(beats, cues),
      wordsPerMinute: pace,
      textSize: size,
      opacity: opacityOf(opacity),
      mirror,
      underWebcam,
      paused,
    });
  }, [beats, cues, pace, size, opacity, mirror, underWebcam, paused]);

  const spoken = spokenText(beats.map((beat) => beat.text));
  const stats = `${wordCount(spoken)} words · ${estimateSeconds(spoken, wordsPerMinute).toFixed(1)} s at ${wordsPerMinute} wpm`;

  function brief(): ScriptBrief {
    const provider = model === "Claude" ? "claude" : "openai";
    return {
      provider,
      model: provider === "claude" ? claudeModel : openaiModel,
      topic,
      language,
      lengthSeconds: lengthSeconds(length),
      wordsPerMinute,
      hookIn3s: chips.includes("Hook in 3 s"),
      screenCues: chips.includes("Screen cues"),
      factCheck: chips.includes("Fact-check notes"),
      cta: chips.includes("CTA"),
    };
  }

  async function generate() {
    const next = brief();
    if (next.model.trim() === "") {
      setNotice("Pick a model in Settings.");
      return;
    }
    if (!onGenerate) {
      setNotice("Generate is unavailable");
      return;
    }
    setNotice("");
    try {
      const result = await onGenerate(next);
      setBeats(result.beats);
      setRawDraft(result.rawDraft);
    } catch (error) {
      setNotice(noticeFrom(error));
    }
  }

  async function regenerate(index: number) {
    const next = brief();
    if (next.model.trim() === "") {
      setNotice("Pick a model in Settings.");
      return;
    }
    if (!onRegenerate) {
      setNotice("Generate is unavailable");
      return;
    }
    setNotice("");
    try {
      const beat = await onRegenerate(next, index, beats);
      setBeats((current) =>
        current.map((item, itemIndex) => (itemIndex === index ? beat : item)),
      );
    } catch (error) {
      setNotice(noticeFrom(error));
    }
  }

  return (
    <div className={`${styles.page} ${styles.columns}`}>
      <Card title="Script" className={styles.scriptCard}>
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
        <Button tone="primary" onClick={() => void generate()}>
          Generate
        </Button>
        <p className={styles.meta} data-raw-draft={rawDraft}>
          {stats}
        </p>
        {notice ? (
          <p className={styles.meta} role="status">
            {notice}
          </p>
        ) : null}
        {beats.map((beat, index) => {
          const label = LABELS[beat.kind] ?? beat.kind;
          return (
            <div key={beat.kind} className={styles.beat}>
              <span className={styles.kind}>{label}</span>
              <span className={styles.meta}>{clock(beat.start)}</span>
              <Button
                className={styles.regen}
                onClick={(event) => {
                  event.preventDefault();
                  void regenerate(index);
                }}
              >
                {`Regenerate ${label}`}
              </Button>
              <input
                className={styles.line}
                value={beat.text}
                aria-label={`${label} beat`}
                onChange={(event) =>
                  setBeats((current) =>
                    current.map((item, itemIndex) =>
                      itemIndex === index
                        ? { ...item, text: event.target.value }
                        : item,
                    ),
                  )
                }
              />
              {beat.cues.map((cue) => (
                <span key={cue} className={`${styles.meta} ${styles.cue}`}>
                  {cue}
                </span>
              ))}
            </div>
          );
        })}
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
              <p className={styles.promptLine}>{beats[0]?.text}</p>
            </div>
          </div>
          <div className={styles.pair}>
            <p className={styles.kicker}>What gets recorded</p>
            <div className={styles.recorded}>
              <p className={styles.banner}>9:16</p>
            </div>
          </div>
        </div>
        <div className={styles.row}>
          {["Screen + cam", "Cam only", "Screen only"].map((item) => (
            <Pill
              key={item}
              selected={preset === item}
              onClick={() => setPreset(item)}
            >
              {item}
            </Pill>
          ))}
          <Button tone="danger">Stop recording</Button>
          <Button
            aria-pressed={paused}
            onClick={() => setPaused((value) => !value)}
          >
            {paused ? "Resume prompter" : "Pause prompter"}
          </Button>
        </div>
        {captureNote ? (
          <p className={styles.meta} role="status">
            {captureNote}
          </p>
        ) : null}
        <SelectField
          label="Display"
          value={display}
          options={["Fixture display"]}
          onChange={setDisplay}
        />
        <SelectField
          label="Camera"
          value={camera}
          options={["Fixture camera"]}
          onChange={setCamera}
        />
        <SelectField
          label="Microphone"
          value={microphone}
          options={["Fixture microphone"]}
          onChange={setMicrophone}
        />
        <div
          className={styles.meter}
          role="meter"
          aria-label="Mic level"
          aria-valuemin={0}
          aria-valuemax={100}
          aria-valuenow={0}
        >
          <span style={{ width: "0%" }} />
        </div>
        <p className={styles.meta}>{`30 s · ${disk.toFixed(1)} MB`}</p>
        <Toggle
          label="Highlight clicks"
          checked={highlightClicks}
          onChange={setHighlightClicks}
        />
        <Toggle
          label="Show keystrokes"
          checked={showKeys}
          onChange={setShowKeys}
        />
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
        <Field label="Speed" value={speed} onChange={setSpeed} />
        <Field label="Text size" value={textSize} onChange={setTextSize} />
        <Field label="Opacity" value={opacity} onChange={setOpacity} />
        <Toggle label="Mirror" checked={mirror} onChange={setMirror} />
        <Toggle
          label="Position under webcam"
          checked={underWebcam}
          onChange={setUnderWebcam}
        />
        <Toggle label="Show screen cues" checked={cues} onChange={setCues} />
        <Toggle label="Hide from recording" checked disabled />
      </Card>
    </div>
  );
}
