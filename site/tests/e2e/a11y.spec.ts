import AxeBuilder from "@axe-core/playwright";
import { expect, test } from "@playwright/test";
import { RESULTS, USER_AGENTS, wizardUrl } from "./helpers.ts";

const TAGS = ["wcag2a", "wcag2aa", "wcag21a", "wcag21aa", "best-practice"];

async function violations(page: import("@playwright/test").Page) {
  await page.waitForLoadState("networkidle");
  const result = await new AxeBuilder({ page }).withTags(TAGS).analyze();
  return result.violations.map((violation) => ({
    id: violation.id,
    help: violation.help,
    nodes: violation.nodes.map((node) => node.target.join(" ")),
  }));
}

for (const [language, folder, locale] of [
  ["en", "", "en-US"],
  ["pt-BR", "/pt-br", "pt-BR"],
] as const) {
  test.describe(`axe in ${language}`, () => {
    test.use({ locale });

    for (const path of [
      "/",
      "/test/",
      "/activate/",
      "/privacy.html",
      "/compatibility.html",
      "/help.html",
    ]) {
      test(`${folder}${path}`, async ({ page }) => {
        await page.goto(`${folder}${path === "/" ? "/" : path}`);
        expect(await violations(page)).toEqual([]);
      });
    }

    test.describe("wizard questions", () => {
      test.use({ userAgent: USER_AGENTS.windowsChrome });
      for (const step of ["system", "browser", "certificate"]) {
        test(step, async ({ page }) => {
          await page.goto(
            wizardUrl({ os: "windows", arch: "x64", browser: "chrome", step }, folder),
          );
          expect(await violations(page)).toEqual([]);
        });
      }
    });

    for (const result of RESULTS) {
      test.describe(result.name, () => {
        test.use({ userAgent: result.agent });
        test("result", async ({ page }) => {
          await page.goto(wizardUrl(result.link, folder));
          expect(await violations(page)).toEqual([]);
        });
      });
    }

    test.describe("phone", () => {
      test.use({ userAgent: USER_AGENTS.iPhone, viewport: { width: 390, height: 844 } });
      test("the notice on the first step", async ({ page }) => {
        await page.goto(wizardUrl({}, folder));
        await expect(page.locator('[data-device-notice="phone"]').first()).toBeVisible();
        expect(await violations(page)).toEqual([]);
      });
    });
  });
}

for (const path of ["/developers.html", "/notices.html", "/404.html"]) {
  test(`${path} (English)`, async ({ page }) => {
    await page.goto(path);
    expect(await violations(page)).toEqual([]);
  });
}
