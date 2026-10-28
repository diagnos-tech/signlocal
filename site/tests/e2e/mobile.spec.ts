import { devices, expect, test } from "@playwright/test";
import { USER_AGENTS, wizardUrl } from "./helpers.ts";

for (const [name, device] of [
  ["iPhone", devices["iPhone 14"]],
  ["Android", devices["Pixel 7"]],
] as const) {
  test.describe(name, () => {
    // The device's default browser would force a new worker; the config already picks Chromium.
    const { defaultBrowserType: _browser, ...options } = device;
    test.use(options);

    test("the wizard explains that signlocal runs on computers and offers the link", async ({
      page,
    }) => {
      await page.goto(wizardUrl());
      await expect(
        page.getByRole("heading", { name: "signlocal runs on computers" }),
      ).toBeVisible();
      await expect(page.getByRole("button", { name: "Copy link" })).toBeVisible();
      await expect(page.getByRole("link", { name: "Email link" })).toHaveAttribute(
        "href",
        /^mailto:\?subject=Install%20signlocal/,
      );
    });

    test("the home page's start card shows the same notice instead of its tiles", async ({
      page,
    }) => {
      await page.goto("/");
      await expect(
        page.getByRole("heading", { name: "signlocal runs on computers" }),
      ).toBeVisible();
      await expect(page.locator("[data-start-tiles]")).toBeHidden();
    });

    test("the navigation bar stays at the bottom and the tiles are big enough to tap", async ({
      page,
    }) => {
      await page.goto(wizardUrl());
      const next = page.getByRole("button", { name: "Next" });
      const box = await next.boundingBox();
      const viewport = page.viewportSize();
      expect(box && viewport && box.y + box.height <= viewport.height).toBe(true);
      for (const tile of await page.locator(".tile").all()) {
        if (!(await tile.isVisible())) continue;
        expect((await tile.boundingBox())?.height ?? 0).toBeGreaterThanOrEqual(44);
      }
    });
  });
}

test.describe("a Chromebook", () => {
  test.use({ userAgent: USER_AGENTS.chromeOs });

  test("says it cannot run signlocal yet, and nothing is preselected", async ({ page }) => {
    await page.goto(wizardUrl());
    await expect(
      page.getByRole("heading", { name: "Chromebooks can't run signlocal yet" }),
    ).toBeVisible();
    await expect(page.getByRole("radio", { name: /Windows/ })).not.toBeChecked();
  });
});
