import { useState } from "react";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import styles from "../screens/screens.module.css";
import { DAILY_STEPS, advance, pausedStep } from "./steps";

export function DailyCard() {
  const [done, setDone] = useState<string[]>([]);
  const pause = pausedStep(done);
  const finished = done.includes("kit");

  return (
    <Card title="Daily Short">
      {DAILY_STEPS.map((step) => (
        <p key={step.id} className={styles.meta}>
          <span className={styles.kicker}>{step.actor}</span> {step.label}
          {done.includes(step.id) ? " · done" : ""}
        </p>
      ))}
      <Button
        tone="primary"
        disabled={finished}
        onClick={() => setDone((current) => advance(current))}
      >
        {finished
          ? "Today is ready"
          : done.length === 0
            ? "Start today"
            : "Continue"}
      </Button>
      <p className={styles.meta}>
        {pause
          ? `Paused for you: ${pause.label}.`
          : finished
            ? "Auto steps are finished."
            : "Start today runs until you need to review."}
      </p>
      <p className={styles.meta}>
        First run: permissions, whisper model, API keys.
      </p>
    </Card>
  );
}
