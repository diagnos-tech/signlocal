import { RESULTS, USER_AGENTS, wizardUrl } from "./helpers.ts";
import { test } from "./test.ts";

// Full-page pictures for the visual review (spec §9.5): saved, not compared. CI uploads
// test-results/screenshots as the `site-screenshots` artifact.
const WIDTHS = [360, 768, 1280, 1440] as const;

// Every page of a width is one test, so it needs more than the default minute.
test.setTimeout(240_000);

for (const [language, folder, locale] of [
  ["en", "", "en-US"],
  ["pt-BR", "/pt-br", "pt-BR"],
] as const) {
  test.describe(`screenshots ${language}`, () => {
    test.use({ locale, userAgent: USER_AGENTS.windowsChrome });

    for (const width of WIDTHS) {
      test(`pages at ${width}`, async ({ page }) => {
        await page.setViewportSize({ width, height: 900 });
        const shots: Array<[string, string]> = [
          ["home", `${folder}/`],
          [
            "wizard-system",
            wizardUrl({ os: "windows", arch: "x64", browser: "chrome", step: "system" }, folder),
          ],
          [
            "wizard-browser",
            wizardUrl({ os: "windows", arch: "x64", browser: "chrome", step: "browser" }, folder),
          ],
          [
            "wizard-certificate",
            wizardUrl(
              { os: "windows", arch: "x64", browser: "chrome", step: "certificate" },
              folder,
            ),
          ],
          ["test", `${folder}/test/`],
          ["compatibility", `${folder}/compatibility.html`],
          ["help", `${folder}/help.html`],
          ["developers", "/developers.html"],
        ];
        for (const result of RESULTS)
          shots.push([`result-${result.name}`, wizardUrl(result.link, folder)]);
        for (const [name, path] of shots) {
          await page.goto(path);
          await page.waitForLoadState("networkidle");
          await page.screenshot({
            path: `test-results/screenshots/${language}-${name}-${width}.png`,
            fullPage: true,
          });
        }
      });
    }
  });
}
