/**
 * Renders the toolbar icon PNGs in public/icons/ from the SVG masters there.
 *
 * `icon.svg` becomes icon-<size>.png for every size the browsers ask for;
 * `icon-template.svg` (black on transparent, for Safari's monochrome toolbar)
 * becomes template-<size>.png. Playwright comes from the e2e package, like in
 * screenshots.ts, so the extension needs no image dependency.
 *
 * Usage: `bun scripts/icons.ts`. Commit the PNGs: builds do not regenerate them.
 */

import { readFileSync, writeFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join, resolve } from "node:path";

const EXTENSION = resolve(import.meta.dirname, "..");
const ICONS = join(EXTENSION, "public", "icons");
const SIZES = [16, 19, 32, 38, 48, 96, 128];
const TEMPLATE_SIZES = [16, 32];

interface Page {
  setViewportSize(size: { width: number; height: number }): Promise<void>;
  setContent(html: string): Promise<void>;
  screenshot(options: { omitBackground: boolean }): Promise<Uint8Array>;
}
interface Browser {
  newPage(): Promise<Page>;
  close(): Promise<void>;
}
interface Chromium {
  launch(options: { channel: string }): Promise<Browser>;
}

async function render(page: Page, svg: string, size: number): Promise<Uint8Array> {
  await page.setViewportSize({ width: size, height: size });
  const uri = `data:image/svg+xml;base64,${Buffer.from(svg).toString("base64")}`;
  await page.setContent(
    `<body style="margin:0;background:transparent"><img src="${uri}" width="${size}" height="${size}" style="display:block"></body>`,
  );
  return page.screenshot({ omitBackground: true });
}

async function main(): Promise<void> {
  const require = createRequire(join(EXTENSION, "..", "e2e", "package.json"));
  const { chromium } = require("@playwright/test") as { chromium: Chromium };
  const browser = await chromium.launch({ channel: "chromium" });
  const page = await browser.newPage();
  const jobs: [string, string, number[]][] = [
    ["icon.svg", "icon", SIZES],
    ["icon-template.svg", "template", TEMPLATE_SIZES],
  ];
  for (const [source, prefix, sizes] of jobs) {
    const svg = readFileSync(join(ICONS, source), "utf8");
    for (const size of sizes) {
      writeFileSync(join(ICONS, `${prefix}-${size}.png`), await render(page, svg, size));
    }
  }
  await browser.close();
  console.log(`icons written to ${ICONS}`);
}

await main();
