import { Button } from "../components/Button";
import { SETTINGS, STEPS, type Route } from "./routes";
import styles from "./TopBar.module.css";

type TopBarProps = {
  projectName: string;
  route: Route;
  onNavigate: (path: string) => void;
};

export function TopBar({ projectName, route, onNavigate }: TopBarProps) {
  return (
    <header className={styles.bar}>
      <p className={styles.mark}>&gt;&gt;</p>
      <h1 className={styles.identity}>sudoakshay/shorts</h1>
      <span className={styles.project}>{projectName}</span>
      <nav className={styles.steps} aria-label="Steps">
        {STEPS.map((step) => (
          <button
            key={step.id}
            type="button"
            className={styles.step}
            aria-current={route.id === step.id ? "page" : undefined}
            onClick={() => onNavigate(step.path)}
          >
            {step.label}
          </button>
        ))}
      </nav>
      <span className={styles.spacer} />
      <button
        type="button"
        className={styles.settings}
        aria-current={route.id === SETTINGS.id ? "page" : undefined}
        onClick={() => onNavigate(SETTINGS.path)}
      >
        {SETTINGS.label}
      </button>
      <Button tone="primary">{route.action}</Button>
    </header>
  );
}
