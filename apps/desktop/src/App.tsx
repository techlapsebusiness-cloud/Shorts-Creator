import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { readHealthName } from "./health";
import { AssetsScreen } from "./screens/AssetsScreen";
import { AudioScreen } from "./screens/AudioScreen";
import { EditScreen } from "./screens/EditScreen";
import { ExportScreen } from "./screens/ExportScreen";
import {
  RecordScreen,
  type GeneratedScript,
  type ScriptBeat,
  type ScriptBrief,
} from "./screens/RecordScreen";
import {
  SettingsScreen,
  type ProviderTestResult,
  type SpendView,
} from "./screens/SettingsScreen";
import { UploadScreen } from "./screens/UploadScreen";
import { applyAccent, type AccentId } from "./shell/accents";
import { routeFromPath, type Route } from "./shell/routes";
import { TopBar } from "./shell/TopBar";
import styles from "./App.module.css";

function testProvider(provider: string, key: string) {
  return invoke<ProviderTestResult>("test_provider", { provider, key });
}

function loadSpend() {
  return invoke<SpendView>("spend_status");
}

function saveCap(capUsd: number) {
  return invoke("set_spend_cap", { capUsd });
}

function generateScript(brief: ScriptBrief) {
  return invoke<GeneratedScript>("generate_script", { brief });
}

function regenerateBeat(
  brief: ScriptBrief,
  index: number,
  beats: ScriptBeat[],
) {
  return invoke<ScriptBeat>("regenerate_beat", { brief, index, beats });
}

export default function App() {
  const [health, setHealth] = useState("booting");
  const [route, setRoute] = useState<Route>(() =>
    routeFromPath(window.location.pathname),
  );
  const [accent, setAccent] = useState<AccentId>("rust");
  const [models, setModels] = useState({ claude: "", openai: "" });

  useEffect(() => {
    let cancelled = false;
    invoke<unknown>("health")
      .then((value) => {
        if (!cancelled) {
          setHealth(readHealthName(value) ?? "unavailable");
        }
      })
      .catch(() => {
        if (!cancelled) {
          setHealth("unavailable");
        }
      });
    return () => {
      cancelled = true;
    };
  }, []);

  useEffect(() => {
    const onPop = () => setRoute(routeFromPath(window.location.pathname));
    window.addEventListener("popstate", onPop);
    return () => window.removeEventListener("popstate", onPop);
  }, []);

  function navigate(path: string) {
    window.history.pushState(null, "", path);
    setRoute(routeFromPath(path));
  }

  function changeAccent(id: AccentId) {
    applyAccent(id);
    setAccent(id);
  }

  return (
    <div className={styles.app}>
      <p className={styles.health} data-testid="health">
        {health}
      </p>
      <TopBar projectName="Example short" route={route} onNavigate={navigate} />
      <main className={styles.stage} data-screen={route.id}>
        {route.id === "record" ? (
          <RecordScreen
            claudeModel={models.claude}
            openaiModel={models.openai}
            onGenerate={generateScript}
            onRegenerate={regenerateBeat}
          />
        ) : null}
        {route.id === "edit" ? <EditScreen /> : null}
        {route.id === "audio" ? <AudioScreen /> : null}
        {route.id === "assets" ? <AssetsScreen /> : null}
        {route.id === "export" ? <ExportScreen /> : null}
        {route.id === "upload" ? <UploadScreen /> : null}
        {route.id === "settings" ? (
          <SettingsScreen
            accent={accent}
            onAccent={changeAccent}
            onTest={testProvider}
            onSpend={loadSpend}
            onCap={saveCap}
            onModel={(provider, model) =>
              setModels((current) => ({ ...current, [provider]: model }))
            }
          />
        ) : null}
      </main>
    </div>
  );
}
