import type { Page } from "@playwright/test";

export async function mockTauri(page: Page) {
  await page.addInitScript(() => {
    const internals = {
      invoke: (cmd: string) => {
        if (cmd === "health") {
          return Promise.resolve({ name: "shorts-studio", version: "0.1.0" });
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
