import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { mockTauri } from "./mockTauri";

const screens = [
  { path: "/", name: "Script", shot: "record" },
  { path: "/edit", name: "Media", shot: "edit" },
  { path: "/audio", name: "Voice", shot: "audio" },
  { path: "/assets", name: "Covers", shot: "assets" },
  { path: "/export", name: "Resolution", shot: "export" },
  { path: "/upload", name: "Upload kit", shot: "upload" },
  { path: "/settings", name: "Providers", shot: "settings" },
];

test.describe("shell screens", () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  for (const screen of screens) {
    test(screen.shot, async ({ page }) => {
      await mockTauri(page);
      await page.goto(screen.path);
      await page.evaluate(async () => {
        await document.fonts.ready;
      });
      await expect(
        page.getByRole("heading", { name: screen.name, exact: true }),
      ).toBeVisible();
      if (screen.shot === "settings") {
        await expect(page.getByTestId("spend")).toHaveText(
          "Spent 0.00 of 20.00 USD.",
        );
      }
      const results = await new AxeBuilder({ page }).analyze();
      const blocked = results.violations.filter(
        (violation) =>
          violation.impact === "serious" || violation.impact === "critical",
      );
      expect(blocked).toEqual([]);
      await expect(page).toHaveScreenshot(`${screen.shot}.png`);
    });
  }
});
