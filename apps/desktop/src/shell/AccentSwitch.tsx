import { ACCENTS, type AccentId } from "./accents";
import styles from "./AccentSwitch.module.css";

type AccentSwitchProps = {
  value: AccentId;
  onChange: (id: AccentId) => void;
};

export function AccentSwitch({ value, onChange }: AccentSwitchProps) {
  return (
    <div className={styles.group} role="radiogroup" aria-label="Accent">
      {ACCENTS.map((accent) => (
        <button
          key={accent.id}
          type="button"
          role="radio"
          className={styles.swatch}
          aria-checked={value === accent.id}
          onClick={() => onChange(accent.id)}
        >
          <span className={styles.chip} style={{ background: accent.value }} />
          {accent.label}
        </button>
      ))}
    </div>
  );
}
