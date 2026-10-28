// Size budgets per page (gzip): the HTML, the JavaScript the page loads (its scripts and every
// chunk they import), the CSS, and the number of font files. The API reference is not checked.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { gzipSync } from "node:zlib";
import { factsOf, type Page } from "./html.ts";

const KB = 1024;

export interface Budget {
  readonly html: number;
  readonly js: number;
  readonly css: number;
  readonly fonts: number;
}

export function budgetFor(path: string): Budget {
  const heavy = /(^|\/)(download\.html|test\/index\.html|activate\/index\.html)$/.test(path);
  const home = /^([a-z-]+\/)?index\.html$/.test(path);
  return {
    html: (path.endsWith("download.html") ? 60 : 40) * KB,
    js: (heavy ? 40 : home ? 12 : 12) * KB,
    css: 30 * KB,
    fonts: 2,
  };
}

const gz = (bytes: Uint8Array | string) => gzipSync(bytes, { level: 9 }).length;

/** The files a module imports statically, as written between quotes after from/import. */
export function importsOf(source: string): string[] {
  return [...source.matchAll(/(?:from|import)\s*["'](\.[^"']+)["']/g)].map(
    (match) => match[1] ?? "",
  );
}

function scriptWeight(dist: string, url: string, seen: Set<string>): number {
  const path = url.startsWith("/") ? url.slice(1) : url;
  if (seen.has(path)) return 0;
  seen.add(path);
  const source = readFileSync(join(dist, path), "utf8");
  const folder = path.includes("/") ? path.slice(0, path.lastIndexOf("/") + 1) : "";
  return (
    gz(source) +
    importsOf(source).reduce(
      (sum, imported) =>
        sum + scriptWeight(dist, new URL(imported, `http://x/${folder}`).pathname.slice(1), seen),
      0,
    )
  );
}

export async function checkBudgets(dist: string, pages: readonly Page[]): Promise<string[]> {
  const problems: string[] = [];
  for (const page of pages) {
    const budget = budgetFor(page.path);
    const facts = await factsOf(page.html);
    const over = (what: string, size: number, limit: number) =>
      size > limit &&
      problems.push(
        `${page.path}: ${what} is ${(size / KB).toFixed(1)} KB gzip; the budget is ${(limit / KB).toFixed(0)} KB`,
      );
    over("the HTML", gz(page.html), budget.html);
    const seen = new Set<string>();
    const scripts = facts.urls.filter((u) => u.tag === "script" && u.value.startsWith("/"));
    over(
      "the JavaScript",
      scripts.reduce((sum, script) => sum + scriptWeight(dist, script.value, seen), 0),
      budget.js,
    );
    const sheets = facts.urls.filter((u) => u.tag === "link" && u.rel === "stylesheet");
    const css = sheets.map((sheet) => readFileSync(join(dist, sheet.value.slice(1)), "utf8"));
    over(
      "the CSS",
      css.reduce((sum, text) => sum + gz(text), 0),
      budget.css,
    );
    const fonts = new Set(
      css.flatMap((text) => [...text.matchAll(/url\(([^)]+\.woff2)\)/g)].map((match) => match[1])),
    );
    if (fonts.size > budget.fonts)
      problems.push(
        `${page.path}: ${fonts.size} font files are referenced; the budget is ${budget.fonts}`,
      );
  }
  return problems;
}
