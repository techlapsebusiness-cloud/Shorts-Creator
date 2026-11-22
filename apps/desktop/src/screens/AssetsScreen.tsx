import { useState } from "react";
import { endBeat, hookFits, nextTitle, TAGLINE } from "../assets/card";
import { Card } from "../components/Card";
import { Pill } from "../components/Pill";
import { Toggle } from "../components/Toggle";
import styles from "./screens.module.css";

const COVERS = [
  ["A", "Hook + face cutout", "This file survives"],
  ["B", "Error hook", "rename failed"],
  ["C", "Before / after", "tmp  ->  json"],
] as const;

const FRAMES = [
  [0, ">>"],
  [0.6, "Next"],
  [1.3, "Follow"],
  [2, TAGLINE],
] as const;
const QUEUE = ["Rename", "Exposure"];
const STYLES = ["Rise", "Glitch", "Slide", "Terminal"] as const;

export function AssetsScreen() {
  const [cover, setCover] = useState("A");
  const [style, setStyle] = useState("Rise");
  const [always, setAlways] = useState(true);

  return (
    <div className={styles.page}>
      <Card title="Covers">
        <div className={styles.covers}>
          {COVERS.map(([id, name, line]) => (
            <button
              key={id}
              type="button"
              className={styles.cover}
              data-selected={cover === id}
              aria-pressed={cover === id}
              onClick={() => setCover(id)}
            >
              <span className={styles.kicker}>
                {id} · {name}
              </span>
              <strong>{line}</strong>
              <span className={styles.meta}>
                {hookFits(line) ? "Fits the frame" : "Overflow"}
              </span>
            </button>
          ))}
        </div>
      </Card>
      <Card title="End card">
        <div className={styles.endRow}>
          {FRAMES.map(([time, frame]) => (
            <div key={frame} className={styles.endFrame}>
              <span className={styles.kicker}>
                {time.toFixed(1)}s · {endBeat(time)}
              </span>
              <strong>{frame}</strong>
            </div>
          ))}
        </div>
        <div className={styles.row}>
          {STYLES.map((item) => (
            <Pill
              key={item}
              selected={style === item}
              onClick={() => setStyle(item)}
            >
              {item}
            </Pill>
          ))}
        </div>
        <p className={styles.meta}>
          Next title: {nextTitle(QUEUE, "Rename")}. Tagline: {TAGLINE}
        </p>
        <Toggle
          label="Add to every new Short"
          checked={always}
          onChange={setAlways}
        />
      </Card>
    </div>
  );
}
