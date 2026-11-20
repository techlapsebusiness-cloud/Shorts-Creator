import { useState } from "react";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Field, SelectField } from "../components/Field";
import { AccentSwitch } from "../shell/AccentSwitch";
import type { AccentId } from "../shell/accents";
import styles from "./screens.module.css";

const JOBS = [
  ["Script", "Claude"],
  ["Upload kit", "Claude"],
  ["Cover text", "ChatGPT"],
  ["Fact-check", "Claude"],
] as const;

const PIPELINE = [
  ["Pick topic", "AI"],
  ["Generate script", "AI"],
  ["Review", "YOU"],
  ["Record with prompter", "YOU"],
  ["Auto-edit", "AUTO"],
  ["Cover and end card", "AUTO"],
  ["Export 4K", "AUTO"],
  ["Upload kit", "AI"],
] as const;

type SettingsScreenProps = {
  accent: AccentId;
  onAccent: (id: AccentId) => void;
};

export function SettingsScreen({ accent, onAccent }: SettingsScreenProps) {
  const [claude, setClaude] = useState("");
  const [openai, setOpenai] = useState("");
  const [claudeModel, setClaudeModel] = useState("Claude placeholder");
  const [openaiModel, setOpenaiModel] = useState("ChatGPT placeholder");
  const [cap, setCap] = useState("20");

  return (
    <div className={`${styles.page} ${styles.split}`}>
      <div className={styles.stack}>
        <Card title="Providers">
          <Field label="Claude key" value={claude} onChange={setClaude} />
          <Button>Test Claude</Button>
          <SelectField
            label="Claude model"
            value={claudeModel}
            options={["Claude placeholder"]}
            onChange={setClaudeModel}
          />
          <Field label="OpenAI key" value={openai} onChange={setOpenai} />
          <Button>Test OpenAI</Button>
          <SelectField
            label="OpenAI model"
            value={openaiModel}
            options={["ChatGPT placeholder"]}
            onChange={setOpenaiModel}
          />
          <p className={styles.meta}>
            Model lists stay as placeholders until the provider module.
          </p>
        </Card>
        <Card title="Spend">
          <Field label="Monthly cap USD" value={cap} onChange={setCap} />
          <p className={styles.meta}>
            Alert at 80 percent. Block at 100 percent.
          </p>
        </Card>
        <Card title="Accent">
          <AccentSwitch value={accent} onChange={onAccent} />
        </Card>
      </div>
      <div className={styles.stack}>
        <Card title="Which AI does what">
          <table className={styles.table}>
            <thead>
              <tr>
                <th>Job</th>
                <th>Provider</th>
              </tr>
            </thead>
            <tbody>
              {JOBS.map(([job, provider]) => (
                <tr key={job}>
                  <td>{job}</td>
                  <td>{provider}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </Card>
        <Card title="Daily Short mode">
          <ol className={styles.steps}>
            {PIPELINE.map(([step, who], index) => (
              <li key={step}>
                <span>
                  {index + 1}. {step}
                </span>
                <span className={styles.who}>{who}</span>
              </li>
            ))}
          </ol>
        </Card>
      </div>
    </div>
  );
}
