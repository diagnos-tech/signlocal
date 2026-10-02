/**
 * Screenshots of every popup state, light and dark, plus the longest state in
 * each locale, into docs/screenshots/popup/ (with SUMMARY.md).
 *
 * The popup is the real build: `wxt build --mode fixtures` compiles
 * src/popup/fixture.ts in, which answers from the query string instead of
 * the background (production builds drop it; checked below). Chromium loads
 * the unpacked build, whose pinned key gives it the known dev ID, and opens
 * chrome-extension://<id>/popup.html directly.
 *
 * Usage: `bun scripts/screenshots.ts [--out <dir>]`. Playwright comes from
 * the e2e package (same pinned version, browsers in PLAYWRIGHT_BROWSERS_PATH).
 */

import { spawnSync } from "node:child_process";
import { mkdirSync, mkdtempSync, readdirSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { tmpdir } from "node:os";
import { join, relative, resolve } from "node:path";
import { EXTENSION_DEV_ID } from "../src/generated/project";
import type { Chromium, Context } from "./playwright-types";

const EXTENSION = resolve(import.meta.dirname, "..");
const REPO = resolve(EXTENSION, "..");
const STATES = ["checking", "ready", "missing", "outdated", "error", "unsupported"] as const;
const THEMES = ["light", "dark"] as const;
/** Browser UI locale → i18n file name. */
const LOCALES: Readonly<Record<string, string>> = {
  en: "en",
  es: "es",
  "pt-PT": "pt-PT",
  "pt-BR": "pt-BR",
  fr: "fr",
  it: "it",
  de: "de",
};
/** The state with the most text, shown in every locale. */
const LOCALE_STATE = "missing";

function arg(name: string): string | undefined {
  const at = process.argv.indexOf(name);
  return at === -1 ? undefined : process.argv[at + 1];
}

function build(mode: "production" | "fixtures"): string {
  const run = spawnSync("bun", ["x", "wxt", "build", "-b", "chrome", "--mode", mode], {
    cwd: EXTENSION,
    stdio: "inherit",
  });
  if (run.status !== 0) throw new Error(`wxt build --mode ${mode} failed`);
  return join(EXTENSION, ".output", mode === "production" ? "chrome-mv3" : "chrome-mv3-fixtures");
}

/** The fixture is the only popup code that reads a query string. */
function assertNoFixtureIn(dir: string): void {
  const chunks = join(dir, "chunks");
  for (const file of readdirSync(chunks)) {
    if (readFileSync(join(chunks, file), "utf8").includes("URLSearchParams")) {
      throw new Error(`production build contains the screenshot fixture: ${file}`);
    }
  }
}

async function open(chromium: Chromium, build: string, locale: string): Promise<Context> {
  const profile = mkdtempSync(join(tmpdir(), "websign-popup-shots-"));
  const context = await chromium.launchPersistentContext(profile, {
    channel: "chromium",
    headless: true,
    locale,
    deviceScaleFactor: 2,
    env: { ...process.env, LANGUAGE: locale.replace("-", "_") },
    args: [`--lang=${locale}`, `--disable-extensions-except=${build}`, `--load-extension=${build}`],
  });
  context.on("close", () => rmSync(profile, { recursive: true, force: true }));
  return context;
}

/** Saves the popup body (its real size) for `state` at `path`. */
async function shoot(context: Context, state: string, theme: string, path: string): Promise<void> {
  const page = await context.newPage();
  await page.setViewportSize({ width: 320, height: 600 });
  await page.emulateMedia({ colorScheme: theme, reducedMotion: "reduce" });
  await page.goto(`chrome-extension://${EXTENSION_DEV_ID}/popup.html?state=${state}&os=win`);
  // Checking appears after 150 ms by design; the others once the card has a title.
  await page.waitForSelector(".status h1", { timeout: 5_000 });
  await page.waitForTimeout(300);
  const box = await page.evaluate(() => {
    const rect = document.body.getBoundingClientRect();
    return { x: 0, y: 0, width: Math.ceil(rect.width), height: Math.ceil(rect.height) };
  });
  await page.screenshot({ path, clip: box });
  await page.close();
}

async function main(): Promise<void> {
  const out = resolve(arg("--out") ?? join(REPO, "docs", "screenshots", "popup"));
  mkdirSync(out, { recursive: true });
  assertNoFixtureIn(build("production"));
  const fixtures = build("fixtures");
  const require = createRequire(join(REPO, "e2e", "package.json"));
  const { chromium } = require("@playwright/test") as { chromium: Chromium };
  const saved: string[] = [];

  const english = await open(chromium, fixtures, "en");
  for (const theme of THEMES) {
    for (const state of STATES) {
      const file = `${state}-${theme}.png`;
      await shoot(english, state, theme, join(out, file));
      saved.push(`- \`${file}\` — ${state}, ${theme} theme, English, Windows.`);
    }
  }
  await english.close();

  for (const [locale, name] of Object.entries(LOCALES)) {
    const context = await open(chromium, fixtures, locale);
    const file = `${LOCALE_STATE}-${name}.png`;
    await shoot(context, LOCALE_STATE, "light", join(out, file));
    saved.push(`- \`${file}\` — ${LOCALE_STATE}, light theme, ${name}.`);
    await context.close();
  }

  const summary = [
    "# docs/screenshots/popup",
    "",
    "The extension popup in every state, generated by `bun scripts/screenshots.ts` in `extension/`",
    "(real build with a fixture instead of the app; 320 px at 2x). Do not edit by hand.",
    "",
    ...saved,
    "",
  ].join("\n");
  writeFileSync(join(out, "SUMMARY.md"), summary);
  console.log(`${saved.length} screenshots in ${relative(REPO, out) || out}`);
}

await main();
