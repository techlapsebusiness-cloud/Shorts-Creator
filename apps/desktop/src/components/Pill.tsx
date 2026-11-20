import styles from "./Pill.module.css";

type PillProps = {
  selected?: boolean;
  onClick?: () => void;
  children: string;
};

export function Pill({ selected = false, onClick, children }: PillProps) {
  return (
    <button
      type="button"
      className={selected ? `${styles.pill} ${styles.on}` : styles.pill}
      aria-pressed={selected}
      onClick={onClick}
    >
      {children}
    </button>
  );
}
