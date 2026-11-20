import { useState } from "react";
import { Card } from "../components/Card";
import { Pill } from "../components/Pill";
import { Toggle } from "../components/Toggle";
import styles from "./screens.module.css";

const COVERS = [
  ["A", "Hook + face cutout", "This file survives"],
  ["B", "Error hook", "rename failed"],
  ["C", "Before / after", "tmp  ->  json"],
] as const;

const FRAMES = [">>", "Next", "Follow", "BUILD. LEARN. REPEAT."] as const;
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
            </button>
          ))}
        </div>
      </Card>
      <Card title="End card">
        <div className={styles.endRow}>
          {FRAMES.map((frame) => (
            <div key={frame} className={styles.endFrame}>
              <span className={styles.kicker}>{frame}</span>
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
        <Toggle
          label="Add to every new Short"
          checked={always}
          onChange={setAlways}
        />
      </Card>
    </div>
  );
}
