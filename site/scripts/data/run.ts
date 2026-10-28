// `bun run data`: writes everything the pages are generated from into src/generated/ (ignored
// by git) and builds public/boot.js. Every generator fails loudly, naming its source.
//
// SIGNLOCAL_SITE_RELEASE picks where the release comes from:
//   github   the GitHub Releases API; any failure stops the build (the Pages deploy).
//   fixture  tests/fixtures/release.json (CI checks, offline work).
//   none     "no release published yet".
//   (unset)  GitHub, else the fixture with a warning (local work).

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { ReleaseData } from "../../src/lib/data-types.ts";
import { buildBoot } from "./boot.ts";
import { readMessageKeys } from "./message-keys.ts";
import { readProject } from "./project.ts";
import { fetchReleaseData } from "./release.ts";
import { readSnippets } from "./snippets.ts";
import { readSupport } from "./support.ts";

const site = fileURLToPath(new URL("../..", import.meta.url)).replace(/\/$/, "");
const repo = `${site}/..`;
const generated = `${site}/src/generated`;
mkdirSync(generated, { recursive: true });

const write = (name: string, content: unknown) =>
  writeFileSync(
    `${generated}/${name}`,
    typeof content === "string" ? content : `${JSON.stringify(content, null, 2)}\n`,
  );

const project = readProject(repo);
write("project.json", project);
write("support.json", readSupport(repo));
write("snippets.json", readSnippets(repo));
write("message-keys.ts", readMessageKeys(site));
await buildBoot(site);

const fixture = () => {
  const data = JSON.parse(
    readFileSync(`${site}/tests/fixtures/release.json`, "utf8"),
  ) as ReleaseData;
  write("release.json", { ...data, source: "fixture" });
};
const github = () =>
  fetchReleaseData({
    repositoryPath: project.repositoryPath,
    slug: project.slug,
    token: process.env.GITHUB_TOKEN,
  });

const mode = process.env.SIGNLOCAL_SITE_RELEASE;
if (mode === "fixture") fixture();
else if (mode === "none")
  write("release.json", {
    source: "fixture",
    generatedAt: new Date().toISOString(),
    release: null,
  });
else if (mode === "github") write("release.json", await github());
else if (mode === undefined || mode === "") {
  try {
    write("release.json", await github());
  } catch (error) {
    console.warn(
      `Release data: GitHub unavailable (${(error as Error).message}); using the sample fixture.`,
    );
    fixture();
  }
} else {
  throw new Error(`SIGNLOCAL_SITE_RELEASE must be github, fixture or none, not "${mode}"`);
}
const release = (JSON.parse(readFileSync(`${generated}/release.json`, "utf8")) as ReleaseData)
  .release;
console.log(`Release data ready: ${release ? release.tag : "no release published"}.`);
