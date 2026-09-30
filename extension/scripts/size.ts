/**
 * Enforces the popup budget of SPEC §4 (docs/ux.md §9): popup.html plus every
 * script and stylesheet it loads, following static imports between chunks,
 * stays under 15 KB uncompressed (what the browser reads from disk).
 *
 * Usage (CI): `bun run build && bun run size`. Exits 1 over budget.
 */

import { existsSync, readFileSync, statSync } from "node:fs";
import { dirname, join, relative } from "node:path";

const BUDGET_BYTES = 15 * 1024;
const OUTPUT = new URL("../.output/chrome-mv3/", import.meta.url).pathname;
const POPUP = join(OUTPUT, "popup.html");

/** Files referenced from HTML (`src`/`href`) or JS (`import … from`, `import(…)`). */
function references(file: string, text: string): string[] {
  const html = /\b(?:src|href)="([^"]+)"/g;
  const js = /\bimport\s*(?:[^"'()]*?from\s*)?\(?\s*["']([^"']+\.(?:js|css))["']/g;
  const pattern = file.endsWith(".html") ? html : js;
  const found: string[] = [];
  for (const [, path = ""] of text.matchAll(pattern)) {
    if (/^[a-z]+:/i.test(path)) continue;
    found.push(path.startsWith("/") ? join(OUTPUT, path) : join(dirname(file), path));
  }
  return found;
}

if (!existsSync(POPUP)) {
  console.error("no chrome build: run `bun run build` first");
  process.exit(1);
}
const seen = new Set<string>();
const queue = [POPUP];
for (let file = queue.pop(); file !== undefined; file = queue.pop()) {
  if (seen.has(file) || !existsSync(file)) continue;
  seen.add(file);
  if (!file.endsWith(".css")) queue.push(...references(file, readFileSync(file, "utf8")));
}
let total = 0;
for (const file of seen) {
  const size = statSync(file).size;
  total += size;
  console.log(`  ${relative(OUTPUT, file)}: ${size} B`);
}
const verdict = total < BUDGET_BYTES ? "ok" : "OVER BUDGET";
console.log(`extension popup: ${total} B (budget ${BUDGET_BYTES} B): ${verdict}`);
if (total >= BUDGET_BYTES) process.exit(1);
