// `bun scripts/check/run.ts [--deploy]`: the post-build gates on dist/. With --deploy (the
// Pages workflow) the build must also be free of the sample-data ribbon.

import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { ERROR_CODES } from "../../src/lib/error-codes.ts";
import { readProject } from "../data/project.ts";
import { checkBudgets } from "./budget.ts";
import { checkCsp } from "./csp.ts";
import { pagesIn } from "./html.ts";
import { checkLinks } from "./links.ts";
import { checkSeo } from "./seo.ts";

const site = fileURLToPath(new URL("../..", import.meta.url)).replace(/\/$/, "");
const dist = `${site}/dist`;
if (!existsSync(dist)) throw new Error("dist/ does not exist: run `bun run build` first");
const { homepage } = readProject(`${site}/..`);
const origin = new URL(homepage).origin;
const pages = pagesIn(dist);

const results: Record<string, string[]> = {
  links: await checkLinks(dist, pages, origin, ERROR_CODES),
  csp: await checkCsp(dist, pages, origin),
  seo: await checkSeo(pages, homepage),
  budget: await checkBudgets(dist, pages),
};
if (process.argv.includes("--deploy")) {
  results.release = pages
    .filter((page) => page.html.includes("Sample release data"))
    .map(
      (page) =>
        `${page.path}: shows the sample-data ribbon; a deploy must use the real release (SIGNLOCAL_SITE_RELEASE=github)`,
    );
}

let failed = false;
for (const [name, problems] of Object.entries(results)) {
  console.log(`${problems.length === 0 ? "ok  " : "FAIL"} ${name}`);
  for (const problem of problems) console.log(`     ${problem}`);
  failed ||= problems.length > 0;
}
console.log(`${pages.length} pages checked.`);
process.exit(failed ? 1 : 0);
