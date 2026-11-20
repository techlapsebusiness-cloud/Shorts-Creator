import type { Page } from "@playwright/test";

export async function mockTauri(page: Page) {
  await page.addInitScript(() => {
    const internals = {
      invoke: (cmd: string) => {
        if (cmd === "health") {
          return Promise.resolve({ name: "shorts-studio", version: "0.1.0" });
        }
        if (cmd === "spend_status") {
          return Promise.resolve({
            spentUsd: 0,
            capUsd: 20,
            decision: "allow",
          });
        }
        if (cmd === "test_provider") {
          return Promise.resolve({
            ok: true,
            message: "Key works",
            models: [],
            defaultModel: "",
          });
        }
        if (cmd === "set_spend_cap") {
          return Promise.resolve(null);
        }
        if (cmd === "generate_script" || cmd === "regenerate_beat") {
          return Promise.resolve({
            rawDraft: "",
            beats: [],
            kind: "hook",
            start: 0,
            text: "",
            cues: [],
          });
        }
        return Promise.reject(new Error(`unmocked command ${cmd}`));
      },
      metadata: {
        currentWindow: { label: "main" },
        currentWebview: { windowLabel: "main", label: "main" },
      },
      transformCallback: () => 0,
      unregisterCallback: () => undefined,
      convertFileSrc: (filePath: string) => filePath,
    };
    Object.defineProperty(window, "__TAURI_INTERNALS__", { value: internals });
  });
}
