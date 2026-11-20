import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { mockTauri } from "./mockTauri";

test("prompter window reads the script and pauses", async ({ page }) => {
  await mockTauri(page);
  await page.goto("/?window=prompter");
  await expect(page.getByText("PROMPTER · NOT RECORDED")).toBeVisible();
  await expect(page.getByText("CUE: point at the folder")).toHaveAttribute(
    "data-cue",
    "true",
  );
  await page.keyboard.press("Space");
  await expect(page.getByRole("status")).toHaveText("Paused");
  const results = await new AxeBuilder({ page }).analyze();
  const blocked = results.violations.filter(
    (violation) =>
      violation.impact === "serious" || violation.impact === "critical",
  );
  expect(blocked).toEqual([]);
});
