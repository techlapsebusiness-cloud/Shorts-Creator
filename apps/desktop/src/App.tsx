import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import styles from "./App.module.css";
import { readHealthName } from "./health";

export default function App() {
  const [health, setHealth] = useState("booting");

  useEffect(() => {
    let cancelled = false;
    invoke<unknown>("health")
      .then((value) => {
        if (cancelled) {
          return;
        }
        setHealth(readHealthName(value) ?? "unavailable");
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

  return (
    <main className={styles.shell}>
      <p className={styles.mark}>&gt;&gt;</p>
      <h1 className={styles.title}>Shorts Studio</h1>
      <p className={styles.status} data-testid="health">
        {health}
      </p>
    </main>
  );
}
