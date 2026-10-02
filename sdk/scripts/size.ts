/**
 * Enforces the size budget of SPEC §13: a site bundling every export of the
 * built main entry pays less than 5 KB minified + gzip. Measured on a consumer
 * bundle, not on `dist/index.js` alone, because that file only re-exports.
 * The optional entries (`/messages`, `/testing`) are reported, not budgeted:
 * sites opt into them, and `/testing` never reaches production.
 *
 * Usage (CI): `bun run build && bun run size`. Exits 1 over budget.
 */

import { gzipSync } from "node:zlib";

const BUDGET_BYTES = 5 * 1024;

async function measure(entry: string): Promise<{ minified: number; gzipped: number }> {
  const result = await Bun.build({
    entrypoints: [new URL(entry, import.meta.url).pathname],
    minify: true,
    target: "browser",
  });
  const [output] = result.outputs;
  if (!result.success || output === undefined) {
    console.error(result.logs.join("\n"));
    process.exit(1);
  }
  const minified = new Uint8Array(await output.arrayBuffer());
  return { minified: minified.length, gzipped: gzipSync(minified, { level: 9 }).length };
}

const main = await measure("./consumer.js");
const verdict = main.gzipped < BUDGET_BYTES ? "ok" : "OVER BUDGET";
console.log(
  `@websign/sdk: ${main.minified} B minified, ${main.gzipped} B gzip (budget ${BUDGET_BYTES} B): ${verdict}`,
);
for (const [name, entry] of [
  ["@websign/sdk/messages", "../dist/messages.js"],
  ["@websign/sdk/testing", "../dist/testing/index.js"],
] as const) {
  const size = await measure(entry);
  console.log(`${name}: ${size.minified} B minified, ${size.gzipped} B gzip (not budgeted)`);
}
if (main.gzipped >= BUDGET_BYTES) process.exit(1);
