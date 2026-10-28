// The pages' Content-Security-Policy allows only this site, with no 'unsafe-inline': so a page
// must have no inline script (besides non-executable JSON data blocks), no <style> element, no
// style attribute and no event-handler attribute, and must load nothing from another origin.

import { existsSync, readdirSync } from "node:fs";
import { join } from "node:path";
import { factsOf, type Page } from "./html.ts";

export const POLICY =
  "default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; manifest-src 'self'; base-uri 'none'; form-action 'self'; object-src 'none'; upgrade-insecure-requests";

const LOADING = new Set([
  "stylesheet",
  "preload",
  "modulepreload",
  "icon",
  "apple-touch-icon",
  "manifest",
]);

export async function checkPage(page: Page, origin: string): Promise<string[]> {
  const problems: string[] = [];
  const facts = await factsOf(page.html);
  const fail = (message: string) => problems.push(`${page.path}: ${message}`);
  if (facts.csp === undefined) fail("has no Content-Security-Policy meta");
  else if (facts.csp !== POLICY)
    fail(`Content-Security-Policy differs from the site's policy: ${facts.csp}`);
  if (facts.inlineStyles > 0) fail(`has ${facts.inlineStyles} <style> element(s)`);
  if (facts.styleAttributes > 0) fail(`has ${facts.styleAttributes} style attribute(s)`);
  if (facts.eventAttributes > 0) fail(`has ${facts.eventAttributes} on* attribute(s)`);
  for (const type of facts.inlineScripts)
    if (type !== "application/json") fail(`has an inline <script type="${type}">`);
  for (const { tag, name, value, rel } of facts.urls) {
    const loads =
      tag === "script" ||
      tag === "img" ||
      tag === "source" ||
      (tag === "link" && LOADING.has(rel ?? ""));
    if (!loads) continue;
    const absolute = /^[a-z][a-z0-9+.-]*:|^\/\//i.test(value);
    if (absolute && new URL(value, origin).origin !== new URL(origin).origin)
      fail(`<${tag} ${name}="${value}"> loads from another origin`);
  }
  return problems;
}

/** SUMMARY.md and README.md are notes for people working on the repository, not for visitors. */
export function checkNotes(dist: string): string[] {
  const found: string[] = [];
  const walk = (folder: string) => {
    for (const entry of readdirSync(folder, { withFileTypes: true })) {
      if (entry.isDirectory()) walk(join(folder, entry.name));
      else if (entry.name === "SUMMARY.md" || entry.name === "README.md")
        found.push(join(folder, entry.name));
    }
  };
  if (existsSync(dist)) walk(dist);
  return found.map((file) => `${file.replace(`${dist}/`, "")}: a folder note was published`);
}

export async function checkCsp(
  dist: string,
  pages: readonly Page[],
  origin: string,
): Promise<string[]> {
  const problems = (await Promise.all(pages.map((page) => checkPage(page, origin)))).flat();
  return [...problems, ...checkNotes(dist)];
}
