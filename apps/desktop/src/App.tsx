import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { readHealthName } from "./health";
import { AssetsScreen } from "./screens/AssetsScreen";
import { AudioScreen } from "./screens/AudioScreen";
import { EditScreen } from "./screens/EditScreen";
import { ExportScreen } from "./screens/ExportScreen";
import { PrompterScreen } from "./screens/PrompterScreen";
import { RecordScreen } from "./screens/RecordScreen";
import {
  SettingsScreen,
  type ProviderTestResult,
  type SpendView,
} from "./screens/SettingsScreen";
import { UploadScreen } from "./screens/UploadScreen";
import { SessionProvider, useSession } from "./session";
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

export default function App() {
  return (
    <SessionProvider>
      <Studio />
    </SessionProvider>
  );
}

function Studio() {
  const session = useSession();
  const [health, setHealth] = useState("booting");
  const [route, setRoute] = useState<Route>(() =>
    routeFromPath(window.location.pathname),
  );
  const [accent, setAccent] = useState<AccentId>("rust");
  const [recordAction, setRecordAction] = useState(0);
  const [prompter] = useState(() => {
    const path = window.location.pathname.replace(/\/$/, "");
    return path === "/prompter";
  });

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

  if (prompter) {
    return <PrompterScreen />;
  }

  return (
    <div className={styles.app}>
      <p className={styles.health} data-testid="health">
        {health}
      </p>
      <TopBar
        projectName={session.project?.name ?? "Example short"}
        route={route}
        onNavigate={navigate}
        onAction={() => {
          if (route.id === "record") {
            setRecordAction((value) => value + 1);
          }
        }}
      />
      <main className={styles.stage} data-screen={route.id}>
        {route.id === "record" ? (
          <RecordScreen
            actionToken={recordAction}
            onOpened={(opened) => {
              session.setOpened(opened);
              navigate("/edit");
            }}
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
          />
        ) : null}
      </main>
    </div>
  );
}
