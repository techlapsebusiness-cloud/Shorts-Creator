import { useState } from "react";
import { Card } from "../components/Card";
import { Pill } from "../components/Pill";
import { SelectField } from "../components/Field";
import { Toggle } from "../components/Toggle";
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

export function EditScreen() {
  const [tab, setTab] = useState<"Layout" | "Captions" | "Motion">("Layout");
  const [position, setPosition] = useState("Bottom right");
  const [size, setSize] = useState("M");
  const [shape, setShape] = useState("Circle");
  const [safeZone, setSafeZone] = useState(true);
  const [removal, setRemoval] = useState(false);
  const [autoZoom, setAutoZoom] = useState(true);
  const [words, setWords] = useState(true);
  const [language, setLanguage] = useState("Hinglish");
  const [tool, setTool] = useState("Select (V)");
  const [snap, setSnap] = useState(true);

  return (
    <div className={styles.page}>
      <div className={styles.columns}>
        <div className={styles.stack}>
          <Card title="Media">
            <p className={styles.meta}>screen.mov</p>
            <p className={styles.meta}>cam.mov</p>
            <p className={styles.meta}>mic.wav</p>
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
              <div className={styles.crop} />
              {safeZone ? <div className={styles.safe} /> : null}
              <div className={styles.cam} />
              <p className={styles.caption}>
                This file <span className={styles.word}>survives</span> a crash.
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
                onClick={() => setPosition(item)}
              />
            ))}
          </div>
          <div className={styles.row}>
            {["S", "M", "L"].map((item) => (
              <Pill
                key={item}
                selected={size === item}
                onClick={() => setSize(item)}
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
                onClick={() => setShape(item)}
              >
                {item}
              </Pill>
            ))}
          </div>
          <Toggle
            label="Background removal"
            checked={removal}
            onChange={setRemoval}
          />
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
        <div className={styles.tools}>
          {TOOLS.map((item) => (
            <button
              key={item}
              type="button"
              className={styles.tool}
              aria-pressed={tool === item}
              onClick={() => setTool(item)}
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
        <ul className={styles.tracks} aria-label="Timeline">
          {TRACKS.map(([name, clip]) => (
            <li key={name} className={styles.track}>
              <span className={styles.trackName}>{name}</span>
              <span className={styles.lane}>
                {clip ? <span className={`${styles.clip} ${clip}`} /> : null}
              </span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  );
}
