import { expect, test } from "@playwright/test";
import { mockTauri } from "./mockTauri";

test("app window boots", async ({ page }) => {
  await mockTauri(page);
  await page.goto("/");
  await expect(
    page.getByRole("heading", { name: "sudoakshay/shorts" }),
  ).toBeVisible();
  await expect(page.getByTestId("health")).toHaveText("shorts-studio");
});
