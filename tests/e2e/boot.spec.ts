import { expect, test } from "@playwright/test";

test("app window boots", async ({ page }) => {
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

  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "Shorts Studio" }),
  ).toBeVisible();
  await expect(page.getByTestId("health")).toHaveText("shorts-studio");
});
