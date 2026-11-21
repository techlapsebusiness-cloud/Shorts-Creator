import { expect, test } from "@playwright/test";
import { mockTauri } from "./mockTauri";

test("split, ripple delete, and undo", async ({ page }) => {
  await mockTauri(page);
  await page.goto("/edit");
  const lane = page.getByTestId("v1-lane");
  await expect(lane).toHaveAttribute("data-count", "1");
  await page.getByRole("button", { name: "Split (S)" }).click();
  await expect(lane).toHaveAttribute("data-count", "2");
  await page.getByRole("button", { name: "Ripple delete" }).click();
  await expect(lane).toHaveAttribute("data-count", "1");
  await page.keyboard.press("Control+z");
  await expect(lane).toHaveAttribute("data-count", "2");
});
