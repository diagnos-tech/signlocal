import { expect, test } from "@playwright/test";
import { allPaths } from "./helpers.ts";

test.describe("every page of every language", () => {
  for (const path of allPaths()) {
    test(path, async ({ page, baseURL }) => {
      const origin = new URL(baseURL ?? "http://localhost").origin;
      const problems: string[] = [];
      page.on("console", (message) => {
        if (message.type() === "error") problems.push(`console: ${message.text()}`);
      });
      page.on("pageerror", (error) => problems.push(`error: ${error.message}`));
      page.on("request", (request) => {
        if (!request.url().startsWith(origin) && !request.url().startsWith("data:"))
          problems.push(`request to ${request.url()}`);
      });
      const response = await page.goto(path);
      expect(response?.status()).toBe(200);
      await page.waitForLoadState("networkidle");
      expect(problems).toEqual([]);
    });

    test(`${path} has no horizontal scroll at 360 px, and the skip link reaches the main content`, async ({
      page,
    }) => {
      await page.setViewportSize({ width: 360, height: 800 });
      await page.goto(path);
      await page.waitForLoadState("networkidle");
      const overflow = await page.evaluate(
        () => document.documentElement.scrollWidth - document.documentElement.clientWidth,
      );
      expect(overflow).toBeLessThanOrEqual(0);
      await page.keyboard.press("Tab");
      await page.keyboard.press("Enter");
      await expect.poll(() => page.evaluate(() => document.activeElement?.id)).toBe("main");
    });
  }
});
