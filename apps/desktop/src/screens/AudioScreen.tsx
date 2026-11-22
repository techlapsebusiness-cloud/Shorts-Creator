import { useState } from "react";
import type { TrackId } from "../bindings/TrackId";
import { useSession } from "../session";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Toggle } from "../components/Toggle";
import styles from "./screens.module.css";

const HEIGHTS = [
  18, 28, 40, 22, 48, 36, 16, 44, 30, 52, 24, 38, 20, 46, 32, 14,
];

const WORDS = ["This", "file", "um", "survives", "a", "crash"];

const STAGES = [
  ["Noise removal", "noise"],
  ["Voice EQ", "eq"],
  ["Compressor", "compressor"],
  ["De-esser", "deesser"],
] as const;

const MIX_TRACK: Record<string, TrackId> = {
  Voice: "A1",
  Music: "A2",
  SFX: "A3",
};

export function AudioScreen() {
  const session = useSession();
  const [enabled, setEnabled] = useState({
    noise: true,
    eq: true,
    compressor: true,
    deesser: false,
  });
  const [amount, setAmount] = useState({
    noise: 60,
    eq: 40,
    compressor: 50,
    deesser: 30,
  });
  const [loudness, setLoudness] = useState(true);
  const [ducking, setDucking] = useState(true);
  const [wet, setWet] = useState(true);
  const [cut, setCut] = useState<number[]>([2]);
  const [mix, setMix] = useState([
    { name: "Voice", gain: 72, mute: false, solo: false },
    { name: "Music", gain: 28, mute: false, solo: false },
    { name: "SFX", gain: 40, mute: false, solo: false },
  ]);

  function writeMix(name: string, patch: { muted?: boolean; solo?: boolean }) {
    const id = MIX_TRACK[name];
    if (!session.project || !id) {
      return;
    }
    session.updateProject({
      ...session.project,
      tracks: session.project.tracks.map((item) =>
        item.id === id ? { ...item, ...patch } : item,
      ),
    });
  }

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
        <Button
          tone="primary"
          onClick={() =>
            setCut(
              WORDS.map((_, index) => index).filter(
                (index) => WORDS[index] === "um",
              ),
            )
          }
        >
          Remove {cut.length === 1 ? "1 cut" : `${cut.length} cuts`}
        </Button>
        <p>
          {WORDS.map((word, index) => {
            const struck = cut.includes(index);
            return (
              <button
                key={`${word}-${index}`}
                type="button"
                className={struck ? styles.struck : styles.wordButton}
                onClick={() =>
                  setCut((current) =>
                    current.includes(index)
                      ? current.filter((item) => item !== index)
                      : [...current, index],
                  )
                }
              >
                {word}
              </button>
            );
          })}
        </p>
        <p className={styles.meta}>
          A struck word cuts picture and sound together, with a 20 ms crossfade.
        </p>
      </Card>
      <div className={styles.split}>
        <Card title="Voice chain">
          <p className={styles.meta}>
            {wet ? "Hearing the chain." : "Hearing the original."} Each stage
            can be bypassed.
          </p>
          <Button tone="quiet" onClick={() => setWet((value) => !value)}>
            {wet ? "A/B original" : "A/B chain"}
          </Button>
          {STAGES.map(([label, key]) => (
            <div key={key} className={styles.stack}>
              <Toggle
                label={label}
                checked={enabled[key]}
                onChange={(value) =>
                  setEnabled((current) => ({ ...current, [key]: value }))
                }
              />
              <label className={styles.meta}>
                Amount {amount[key]}
                <input
                  type="range"
                  min={0}
                  max={100}
                  value={amount[key]}
                  aria-label={`${label} amount`}
                  onChange={(event) =>
                    setAmount((current) => ({
                      ...current,
                      [key]: Number(event.target.value),
                    }))
                  }
                />
              </label>
            </div>
          ))}
          <Toggle
            label="Loudness -14 LUFS"
            checked={loudness}
            onChange={setLoudness}
          />
        </Card>
        <Card title="Mixer">
          <div className={styles.mixer}>
            {mix.map((track, index) => (
              <div key={track.name}>
                <p className={styles.kicker}>{track.name}</p>
                <div className={styles.fader}>
                  <span style={{ width: `${track.mute ? 0 : track.gain}%` }} />
                </div>
                <div className={styles.row}>
                  <Button
                    tone={track.mute ? "danger" : "quiet"}
                    onClick={() => {
                      const mute = !track.mute;
                      setMix((current) =>
                        current.map((item, itemIndex) =>
                          itemIndex === index ? { ...item, mute } : item,
                        ),
                      );
                      writeMix(track.name, { muted: mute });
                    }}
                  >
                    Mute
                  </Button>
                  <Button
                    tone={track.solo ? "primary" : "quiet"}
                    onClick={() => {
                      const solo = !track.solo;
                      setMix((current) =>
                        current.map((item, itemIndex) =>
                          itemIndex === index ? { ...item, solo } : item,
                        ),
                      );
                      writeMix(track.name, { solo });
                    }}
                  >
                    Solo
                  </Button>
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
