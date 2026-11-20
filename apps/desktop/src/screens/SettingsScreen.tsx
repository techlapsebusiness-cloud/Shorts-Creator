import { useEffect, useState } from "react";
import { Button } from "../components/Button";
import { Card } from "../components/Card";
import { Field, SelectField } from "../components/Field";
import { AccentSwitch } from "../shell/AccentSwitch";
import type { AccentId } from "../shell/accents";
import styles from "./screens.module.css";

const EMPTY_MODELS = "No models yet";

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

export type ProviderTestResult = {
  ok: boolean;
  message: string;
  models: string[];
  defaultModel: string;
};

export type SpendView = {
  spentUsd: number;
  capUsd: number;
  decision: string;
};

type SettingsScreenProps = {
  accent: AccentId;
  onAccent: (id: AccentId) => void;
  onTest?: (provider: string, key: string) => Promise<ProviderTestResult>;
  onSpend?: () => Promise<SpendView>;
  onCap?: (capUsd: number) => void;
  onModel?: (provider: "claude" | "openai", model: string) => void;
};

function spendLine(spent: SpendView | null, ready: boolean): string {
  if (!ready) {
    return "Spent …";
  }
  if (!spent) {
    return "Spend unavailable";
  }
  const base = `Spent ${spent.spentUsd.toFixed(2)} of ${spent.capUsd.toFixed(2)} USD.`;
  if (spent.decision === "alert") {
    return `${base} Cap alert.`;
  }
  if (spent.decision === "block") {
    return `${base} Cap reached.`;
  }
  return base;
}

function chosenModel(result: ProviderTestResult): string[] {
  if (result.models.length === 0) {
    return [EMPTY_MODELS];
  }
  return result.models;
}

function selectedModel(models: string[], preferred: string): string {
  if (preferred && models.includes(preferred)) {
    return preferred;
  }
  return models[0] ?? EMPTY_MODELS;
}

export function SettingsScreen({
  accent,
  onAccent,
  onTest,
  onSpend,
  onCap,
  onModel,
}: SettingsScreenProps) {
  const [claude, setClaude] = useState("");
  const [openai, setOpenai] = useState("");
  const [claudeModels, setClaudeModels] = useState([EMPTY_MODELS]);
  const [openaiModels, setOpenaiModels] = useState([EMPTY_MODELS]);
  const [claudeModel, setClaudeModel] = useState(EMPTY_MODELS);
  const [openaiModel, setOpenaiModel] = useState(EMPTY_MODELS);
  const [cap, setCap] = useState("20");
  const [spent, setSpent] = useState<SpendView | null>(null);
  const [spendReady, setSpendReady] = useState(false);
  const [notice, setNotice] = useState("");

  useEffect(() => {
    if (!onSpend) {
      return;
    }
    let cancelled = false;
    onSpend()
      .then((status) => {
        if (!cancelled) {
          setSpent(status);
          setCap(String(status.capUsd));
          setSpendReady(true);
        }
      })
      .catch(() => {
        if (!cancelled) {
          setSpent(null);
          setSpendReady(true);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [onSpend]);

  async function runTest(provider: "claude" | "openai") {
    if (!onTest) {
      setNotice("Test is unavailable");
      return;
    }
    const key = provider === "claude" ? claude : openai;
    setNotice("");
    try {
      const result = await onTest(provider, key);
      setNotice(result.message);
      const models = chosenModel(result);
      const selected = selectedModel(models, result.defaultModel);
      const chosen = selected === EMPTY_MODELS ? "" : selected;
      onModel?.(provider, chosen);
      if (provider === "claude") {
        setClaudeModels(models);
        setClaudeModel(selected);
        if (result.ok) {
          setClaude("");
        }
      } else {
        setOpenaiModels(models);
        setOpenaiModel(selected);
        if (result.ok) {
          setOpenai("");
        }
      }
    } catch {
      setNotice("Test failed");
    }
  }

  function commitCap() {
    const value = Number(cap);
    if (Number.isFinite(value) && value > 0) {
      onCap?.(value);
    }
  }

  return (
    <div className={`${styles.page} ${styles.split}`}>
      <div className={styles.stack}>
        <Card title="Providers">
          <Field
            label="Claude key"
            value={claude}
            onChange={setClaude}
            secret
          />
          <Button onClick={() => void runTest("claude")}>Test Claude</Button>
          <SelectField
            label="Claude model"
            value={claudeModel}
            options={claudeModels}
            onChange={(value) => {
              setClaudeModel(value);
              onModel?.("claude", value === EMPTY_MODELS ? "" : value);
            }}
          />
          <Field
            label="OpenAI key"
            value={openai}
            onChange={setOpenai}
            secret
          />
          <Button onClick={() => void runTest("openai")}>Test OpenAI</Button>
          <SelectField
            label="OpenAI model"
            value={openaiModel}
            options={openaiModels}
            onChange={(value) => {
              setOpenaiModel(value);
              onModel?.("openai", value === EMPTY_MODELS ? "" : value);
            }}
          />
          <p className={styles.meta}>
            Models load from the provider when you press Test.
          </p>
          {notice ? (
            <p className={styles.meta} role="status">
              {notice}
            </p>
          ) : null}
        </Card>
        <Card title="Spend">
          <Field
            label="Monthly cap USD"
            value={cap}
            onChange={setCap}
            onBlur={commitCap}
          />
          <p className={styles.meta} data-testid="spend">
            {spendLine(spent, spendReady)}
          </p>
          <p className={styles.meta}>
            Alert at 80 percent. Block at 100 percent. Token prices live in
            config and stay at zero until filled in.
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
