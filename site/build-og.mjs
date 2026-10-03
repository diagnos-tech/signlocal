// Regenerates site/assets/og.png: the 1200x630 social-share card used by the
// Open Graph and Twitter meta tags in the site's pages. The card's wording is
// read from project.toml (the single source of truth for the product name and
// tagline), so renaming the product and re-running this script is enough to
// keep the card in sync. The image is committed so the site works straight
// from a checkout; rerun this after the name or tagline changes.
//
// Run it from the repository root after `bun install`:
//
//   bun run site/build-og.mjs
//
// Playwright ships only as the e2e workspace's dev dependency, so we resolve it
// from there rather than from the site's own (empty) dependency tree. Chromium
// is launched headless; PLAYWRIGHT_BROWSERS_PATH already points at the
// pre-installed browser in CI and in the cloud dev environment.
import { createRequire } from "node:module";
import { fileURLToPath, pathToFileURL } from "node:url";
import { dirname, join } from "node:path";
import { readFileSync, writeFileSync } from "node:fs";

const SITE_DIR = dirname(fileURLToPath(import.meta.url));
const ROOT_DIR = dirname(SITE_DIR);

// Resolve @playwright/test as if required from the e2e workspace.
const requireFromE2e = createRequire(join(ROOT_DIR, "e2e", "package.json"));
const { chromium } = await import(
  pathToFileURL(requireFromE2e.resolve("@playwright/test")).href
);
const OUT = join(SITE_DIR, "assets", "og.png");
const WIDTH = 1200;
const HEIGHT = 630;

/** Reads a top-level `key = "value"` string from project.toml. */
function fromProject(key) {
  const toml = readFileSync(join(ROOT_DIR, "project.toml"), "utf8");
  const match = toml.match(new RegExp(`^\\s*${key}\\s*=\\s*"([^"]*)"`, "m"));
  if (!match) throw new Error(`project.toml is missing ${key}`);
  return match[1];
}

const name = fromProject("name");
const tagline = fromProject("tagline");

// The brand mark (the signature wave), inlined from assets/icon.svg so the card
// has no external dependencies when Chromium renders it from a data URL.
const mark = `<svg viewBox="0 0 24 24" fill="none" stroke="#9DA8FF" stroke-width="2"
  stroke-linecap="round" stroke-linejoin="round" width="52" height="52">
  <path d="M3 20c4 0 5-9 8-9s1 6 4 6 3-4 6-4"/><path d="M4 12l4-4 3 3-4 4z"/></svg>`;

// System fonts only (no network), matching the site's --ws-font stack. The
// palette mirrors assets/tokens.css (dark scheme) so the card reads as part of
// the product.
const html = `<!doctype html><html><head><meta charset="utf-8"><style>
  *{margin:0;box-sizing:border-box}
  html,body{width:${WIDTH}px;height:${HEIGHT}px}
  body{display:flex;flex-direction:column;justify-content:space-between;
    padding:72px;color:#E7E9EF;overflow:hidden;position:relative;
    font-family:system-ui,-apple-system,"Segoe UI",Roboto,"Helvetica Neue",sans-serif;
    background:radial-gradient(1100px 620px at 82% -12%,#1A1F3C 0%,#0E1016 60%)}
  .glow{position:absolute;inset:0;background:
    radial-gradient(420px 420px at 88% 24%,rgba(78,94,228,.28),transparent 70%)}
  .row{display:flex;align-items:center;gap:18px;position:relative}
  .brand{font-size:34px;font-weight:700;letter-spacing:-.01em}
  .eyebrow{color:#9DA8FF;font-size:19px;font-weight:600;letter-spacing:.2em;
    text-transform:uppercase;margin-bottom:22px}
  h1{font-size:58px;line-height:1.1;font-weight:750;max-width:1000px;
    letter-spacing:-.02em;color:#F3F4F8}
  .bar{width:96px;height:6px;border-radius:3px;background:#4E5EE4;margin:26px 0 0}
  .meta{position:relative;color:#A3AAB9;font-size:23px;line-height:1.5}
  .meta b{color:#E7E9EF;font-weight:600}
</style></head><body>
  <div class="glow"></div>
  <div class="row"><div>${mark}</div><div class="brand">${name}</div></div>
  <div>
    <div class="eyebrow">Free &middot; Open Source</div>
    <h1>${tagline}</h1>
    <div class="bar"></div>
  </div>
  <div class="meta">
    <div>Chrome &middot; Edge &middot; Firefox &middot; Opera &middot; Brave &middot; Safari</div>
    <div><b>Windows</b> &middot; <b>macOS</b> &middot; <b>Linux</b></div>
  </div>
</body></html>`;

// Default to Playwright's own managed browser (`npx playwright install`). In
// CI or sandboxes where a Chromium is pre-provisioned elsewhere, point
// WEBSIGN_CHROMIUM at its executable to skip the download.
const executablePath = process.env.WEBSIGN_CHROMIUM || undefined;
const browser = await chromium.launch({ executablePath });
try {
  const page = await browser.newPage({
    viewport: { width: WIDTH, height: HEIGHT },
    deviceScaleFactor: 1,
  });
  await page.setContent(html, { waitUntil: "networkidle" });
  const png = await page.screenshot({ type: "png" });
  writeFileSync(OUT, png);
  console.log(`wrote ${OUT} (${png.length} bytes, ${WIDTH}x${HEIGHT})`);
} finally {
  await browser.close();
}
