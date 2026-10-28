// src/client/boot.ts → public/boot.js: the small blocking script every page loads first.

const BUDGET_BYTES = 1024;

export async function buildBoot(site: string): Promise<void> {
  const result = await Bun.build({
    entrypoints: [`${site}/src/client/boot.ts`],
    outdir: `${site}/public`,
    naming: "boot.js",
    minify: true,
    target: "browser",
    format: "iife",
  });
  if (!result.success) throw new Error(`boot.ts did not build:\n${result.logs.join("\n")}`);
  const output = result.outputs[0];
  if (!output) throw new Error("boot.ts produced no output");
  const size = (await output.arrayBuffer()).byteLength;
  if (size > BUDGET_BYTES)
    throw new Error(`public/boot.js is ${size} bytes minified; the budget is ${BUDGET_BYTES}`);
}
