// Every link and file reference of every built page must resolve the way GitHub Pages serves
// them, every #fragment must exist in its target, nothing may be plain http, and every address
// other code links to (docs of the paths contract, the SDK's error anchors) must exist.

import { existsSync, readFileSync } from "node:fs";
import { join } from "node:path";
import { resolve } from "../serve.ts";
import { type Facts, factsOf, type Page } from "./html.ts";

/** Pages the SDK, the extension popup and the app link to (spec §1.3). */
export const CONTRACT_PATHS = [
  "download.html",
  "activate/",
  "test/",
  "privacy.html",
  "developers.html",
  "api/",
] as const;

/** Where a link points: a file of dist/ (with its fragment), another site, or nowhere. */
export function targetOf(
  dist: string,
  fromPath: string,
  url: string,
  origin: string,
): { file: string; fragment: string } | "external" | undefined {
  if (/^(mailto|signlocal|tel):/.test(url)) return "external";
  const absolute = new URL(url, new URL(`/${fromPath}`, origin));
  if (absolute.origin !== origin) return "external";
  let found = resolve(dist, decodeURIComponent(absolute.pathname));
  if (found && "redirect" in found) found = resolve(dist, found.redirect);
  return found && "file" in found
    ? { file: found.file, fragment: absolute.hash.slice(1) }
    : undefined;
}

export async function checkLinks(
  dist: string,
  pages: readonly Page[],
  origin: string,
  errorCodes: readonly string[],
): Promise<string[]> {
  const problems: string[] = [];
  const cache = new Map<string, Facts>();
  const factsFor = async (file: string) => {
    let facts = cache.get(file);
    if (!facts) {
      facts = await factsOf(readFileSync(file, "utf8"));
      cache.set(file, facts);
    }
    return facts;
  };
  for (const page of pages) {
    const facts = await factsFor(join(dist, page.path));
    for (const { tag, name, value } of facts.urls) {
      if (value === "") continue;
      if (/^http:\/\//i.test(value))
        problems.push(`${page.path}: <${tag} ${name}="${value}"> is plain http`);
      const target = targetOf(dist, page.path, value, origin);
      if (target === "external") continue;
      if (!target) {
        problems.push(`${page.path}: <${tag} ${name}="${value}"> does not resolve`);
        continue;
      }
      if (target.fragment && !(await factsFor(target.file)).ids.has(target.fragment)) {
        problems.push(
          `${page.path}: ${value} points to #${target.fragment}, which ${target.file.replace(`${dist}/`, "")} does not have`,
        );
      }
    }
  }
  for (const path of CONTRACT_PATHS) {
    const found = resolve(dist, `/${path}`);
    if (!found || "redirect" in found) problems.push(`The contract path /${path} does not exist`);
  }
  const developers = join(dist, "developers.html");
  if (existsSync(developers)) {
    const { ids } = await factsFor(developers);
    for (const code of errorCodes)
      if (!ids.has(`error-${code}`)) problems.push(`developers.html has no #error-${code}`);
  }
  return problems;
}
