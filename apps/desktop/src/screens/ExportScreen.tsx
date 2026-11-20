import { useState } from "react";
import { Card } from "../components/Card";
import { Toggle } from "../components/Toggle";
import styles from "./screens.module.css";

const RESOLUTIONS = [
  ["4K", "2160 x 3840"],
  ["1440p", "1440 x 2560"],
  ["1080p", "1080 x 1920"],
] as const;

export function ExportScreen() {
  const [resolution, setResolution] = useState("4K");
  const [ease, setEase] = useState(true);
  const [sixty, setSixty] = useState(true);
  const [blur, setBlur] = useState(false);
  const [cursor, setCursor] = useState(true);
  const [blend, setBlend] = useState(true);
  const [cover, setCover] = useState(true);
  const [srt, setSrt] = useState(true);
  const [meta, setMeta] = useState(true);

  return (
    <div className={styles.page}>
      <Card title="Resolution">
        <div className={styles.cards}>
          {RESOLUTIONS.map(([name, size]) => (
            <button
              key={name}
              type="button"
              className={styles.choice}
              aria-pressed={resolution === name}
              onClick={() => setResolution(name)}
            >
              <strong>{name}</strong>
              <span className={styles.meta}>{size}</span>
            </button>
          ))}
        </div>
        <p className={styles.meta}>
          30 fps · h264 · 40000 kbps video · about 148 MB
        </p>
      </Card>
      <div className={styles.split}>
        <Card title="Motion smoothness">
          <Toggle label="Ease in out" checked={ease} onChange={setEase} />
          <Toggle
            label="Animate at 60 fps"
            checked={sixty}
            onChange={setSixty}
          />
          <Toggle
            label="Motion blur on fast moves"
            checked={blur}
            onChange={setBlur}
          />
          <Toggle
            label="Smooth cursor path"
            checked={cursor}
            onChange={setCursor}
          />
          <Toggle
            label="Frame blending on speed-ups"
            checked={blend}
            onChange={setBlend}
          />
        </Card>
        <Card title="Also export">
          <Toggle label="Cover PNG" checked={cover} onChange={setCover} />
          <Toggle label="Captions .srt" checked={srt} onChange={setSrt} />
          <Toggle label="Metadata .txt" checked={meta} onChange={setMeta} />
        </Card>
      </div>
      <Card title="Pre-flight">
        <ul className={styles.checks}>
          {[
            "Vertical 9:16",
            "Under 60 seconds",
            "Captions present",
            "Safe zone on",
          ].map((check) => (
            <li key={check}>
              {check}
              <span className={styles.pass}>Pass</span>
            </li>
          ))}
        </ul>
        <p className={styles.meta}>Render queue · Example short</p>
        <div className={styles.queue} aria-hidden="true">
          <span />
        </div>
      </Card>
    </div>
  );
}
