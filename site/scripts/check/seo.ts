// Search and sharing basics on every built page: one h1, a title and description that are
// unique per page and language, a canonical address under the homepage, the full set of language
// alternates on translated pages, a lang attribute that matches the folder, and noindex only on
// /activate/.

import { LOCALES } from "../../src/i18n/locales.ts";
import { factsOf, type Page } from "./html.ts";

/** "pt-br/download.html" → the language its folder says; English when there is no folder. */
export function languageOf(path: string): string {
  const folder = path.split("/")[0] ?? "";
  return LOCALES.find((locale) => locale.segment !== "" && locale.segment === folder)?.code ?? "en";
}

const ENGLISH_ONLY = new Set(["developers.html", "notices.html", "404.html"]);

export async function checkSeo(pages: readonly Page[], homepage: string): Promise<string[]> {
  const problems: string[] = [];
  const titles = new Map<string, string>();
  const descriptions = new Map<string, string>();
  for (const page of pages) {
    const facts = await factsOf(page.html);
    const fail = (message: string) => problems.push(`${page.path}: ${message}`);
    if (facts.h1Count !== 1) fail(`has ${facts.h1Count} <h1> (exactly one is required)`);
    if (!facts.title) fail("has no <title>");
    const description = facts.meta.description;
    if (!description) fail("has no meta description");
    if (!facts.canonical?.startsWith(homepage))
      fail(`canonical "${facts.canonical}" is not under ${homepage}`);
    const language = languageOf(page.path);
    if (facts.lang !== language) fail(`lang="${facts.lang}" but the page is in ${language}`);
    const noindex = (facts.meta.robots ?? "").includes("noindex");
    const activate = page.path.endsWith("activate/index.html");
    if (activate !== noindex) fail(activate ? "/activate/ must have noindex" : "has noindex");
    const translated = !ENGLISH_ONLY.has(page.path) && page.path !== "404.html";
    if (translated) {
      const expected = [...LOCALES.map((locale) => locale.code), "x-default"].sort();
      const actual = facts.alternates.map((alternate) => alternate.hreflang).sort();
      if (expected.join() !== actual.join()) fail(`hreflang set is ${actual.join(", ")}`);
      const key = `${language}|${facts.title}`;
      if (titles.has(key)) fail(`title "${facts.title}" is also the title of ${titles.get(key)}`);
      titles.set(key, page.path);
      const dkey = `${language}|${description}`;
      if (description && descriptions.has(dkey))
        fail(`description is also the description of ${descriptions.get(dkey)}`);
      if (description) descriptions.set(dkey, page.path);
    }
  }
  return problems;
}
