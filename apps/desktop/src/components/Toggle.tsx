import styles from "./Toggle.module.css";

type ToggleProps = {
  label: string;
  checked: boolean;
  disabled?: boolean;
  onChange?: (checked: boolean) => void;
};

export function Toggle({ label, checked, disabled, onChange }: ToggleProps) {
  return (
    <button
      type="button"
      role="switch"
      className={styles.toggle}
      aria-checked={checked}
      disabled={disabled}
      onClick={() => onChange?.(!checked)}
    >
      <span className={checked ? `${styles.track} ${styles.on}` : styles.track}>
        <span className={styles.knob} />
      </span>
      {label}
    </button>
  );
}
