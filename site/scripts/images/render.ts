// `bun run images`: renders the social preview and the home-screen icon from their HTML
// sources into public/. Run by hand when the mark or the tagline changes; the PNGs are committed.
// SIGNLOCAL_CHROME points at a Chrome or Chromium binary when Playwright's own is not installed.

import { fileURLToPath } from "node:url";
import { chromium } from "@playwright/test";

const here = (path: string) => fileURLToPath(new URL(path, import.meta.url));

const executablePath = process.env.SIGNLOCAL_CHROME;
const browser = await chromium.launch(executablePath ? { executablePath } : {});
const targets = [
  { source: "og.html", out: "../../public/og.png", width: 1200, height: 630 },
  { source: "touch-icon.html", out: "../../public/apple-touch-icon.png", width: 180, height: 180 },
] as const;

for (const target of targets) {
  const page = await browser.newPage({ viewport: { width: target.width, height: target.height } });
  await page.goto(`file://${here(target.source)}`);
  await page.evaluate(() => document.fonts.ready);
  await page.screenshot({ path: here(target.out) });
  await page.close();
  console.log(`wrote ${target.out.replace("../../", "site/")}`);
}
await browser.close();
