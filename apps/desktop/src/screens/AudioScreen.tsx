import { useState } from "react";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Toggle } from "../components/Toggle";
import styles from "./screens.module.css";

const HEIGHTS = [
  18, 28, 40, 22, 48, 36, 16, 44, 30, 52, 24, 38, 20, 46, 32, 14,
];

export function AudioScreen() {
  const [noise, setNoise] = useState(true);
  const [eq, setEq] = useState(true);
  const [compressor, setCompressor] = useState(true);
  const [deesser, setDeesser] = useState(false);
  const [loudness, setLoudness] = useState(true);
  const [ducking, setDucking] = useState(true);

  return (
    <div className={styles.page}>
      <Card title="Voice">
        <div className={styles.wave} aria-hidden="true">
          {HEIGHTS.map((height, index) => (
            <span
              key={index}
              className={
                index === 4
                  ? styles.mark
                  : index === 9
                    ? styles.cut
                    : styles.bar
              }
              style={{ height }}
            />
          ))}
        </div>
        <p className={styles.meta}>
          Amber marks are fillers. Red marks are silence.
        </p>
        <Button tone="primary">Remove 3 cuts</Button>
        <p>
          This file <del className={styles.struck}>um</del> survives a crash
          when the temp file is renamed.
        </p>
      </Card>
      <div className={styles.split}>
        <Card title="Voice chain">
          <p className={styles.meta}>One click, each stage can be bypassed.</p>
          <Toggle label="Noise removal" checked={noise} onChange={setNoise} />
          <Toggle label="Voice EQ" checked={eq} onChange={setEq} />
          <Toggle
            label="Compressor"
            checked={compressor}
            onChange={setCompressor}
          />
          <Toggle label="De-esser" checked={deesser} onChange={setDeesser} />
          <Toggle
            label="Loudness -14 LUFS"
            checked={loudness}
            onChange={setLoudness}
          />
        </Card>
        <Card title="Mixer">
          <div className={styles.mixer}>
            {[
              ["Voice", "72%"],
              ["Music", "28%"],
              ["SFX", "40%"],
            ].map(([name, width]) => (
              <div key={name}>
                <p className={styles.kicker}>{name}</p>
                <div className={styles.fader}>
                  <span style={{ width }} />
                </div>
              </div>
            ))}
          </div>
          <Toggle
            label="Auto-ducking"
            checked={ducking}
            onChange={setDucking}
          />
        </Card>
      </div>
    </div>
  );
}
