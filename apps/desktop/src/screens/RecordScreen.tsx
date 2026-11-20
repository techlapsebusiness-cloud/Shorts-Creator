import { useState } from "react";
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

export function RecordScreen() {
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
  const [preset, setPreset] = useState("Screen + cam");

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
          <Button>Pause prompter</Button>
        </div>
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
      </Card>
    </div>
  );
}
