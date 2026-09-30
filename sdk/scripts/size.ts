/**
 * Enforces the size budget of SPEC §11: a site bundling every export of the
 * built `dist/` pays less than 5 KB minified + gzip. Measured on a consumer
 * bundle, not on `dist/index.js` alone, because that file only re-exports.
 *
 * Usage (CI): `bun run build && bun run size`. Exits 1 over budget.
 */

import { gzipSync } from "node:zlib";

const BUDGET_BYTES = 5 * 1024;

const result = await Bun.build({
  entrypoints: [new URL("./consumer.js", import.meta.url).pathname],
  minify: true,
  target: "browser",
});
const [output] = result.outputs;
if (!result.success || output === undefined) {
  console.error(result.logs.join("\n"));
  process.exit(1);
}
const minified = new Uint8Array(await output.arrayBuffer());
const gzipped = gzipSync(minified, { level: 9 }).length;
const verdict = gzipped < BUDGET_BYTES ? "ok" : "OVER BUDGET";
console.log(
  `@websign/sdk: ${minified.length} B minified, ${gzipped} B gzip (budget ${BUDGET_BYTES} B): ${verdict}`,
);
if (gzipped >= BUDGET_BYTES) process.exit(1);
