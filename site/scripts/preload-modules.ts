// After the build: lists every module a page imports in its <head>. Astro puts a page's scripts
// at the end of <body>, and the modules they import are only discovered once those have been
// downloaded; naming the whole graph up front loads it in one round trip instead of two.

import { readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { fileURLToPath } from "node:url";

const dist = join(fileURLToPath(new URL("..", import.meta.url)), "dist");

/** The static imports of a bundled module (dynamic `import()` calls stay lazy). */
const IMPORT = /\b(?:from|import)\s*["'](\.{1,2}\/[^"']+\.js)["']/g;

/** Every module reachable from `entry` through static imports, nearest first, entry excluded. */
export function dependencies(entry: string, read: (path: string) => string): string[] {
  const seen = new Set<string>([entry]);
  const queue = [entry];
  const found: string[] = [];
  while (queue.length > 0) {
    const current = queue.shift() as string;
    for (const match of read(current).matchAll(IMPORT)) {
      const url = new URL(match[1] as string, `http://x${current}`).pathname;
      if (seen.has(url)) continue;
      seen.add(url);
      queue.push(url);
      found.push(url);
    }
  }
  return found;
}

/** `html` with a modulepreload hint for each dependency of its module scripts, after the CSS. */
export function addPreloads(html: string, read: (path: string) => string): string {
  if (html.includes('rel="modulepreload"')) return html;
  const entries = [...html.matchAll(/<script type="module" src="([^"]+)"/g)].map(
    (match) => match[1] as string,
  );
  const wanted = new Set<string>();
  for (const entry of entries) for (const url of dependencies(entry, read)) wanted.add(url);
  for (const entry of entries) wanted.delete(entry);
  if (wanted.size === 0) return html;
  const links = [...wanted].map((url) => `<link rel="modulepreload" href="${url}">`).join("");
  const anchor = html.match(/<link rel="stylesheet"[^>]*>/);
  if (!anchor || anchor.index === undefined) return html;
  const at = anchor.index + anchor[0].length;
  return html.slice(0, at) + links + html.slice(at);
}

function pages(folder: string): string[] {
  return readdirSync(folder, { withFileTypes: true }).flatMap((entry) => {
    const path = join(folder, entry.name);
    if (entry.isDirectory()) return pages(path);
    return entry.name.endsWith(".html") ? [path] : [];
  });
}

if (import.meta.main) {
  const read = (url: string) => readFileSync(join(dist, url), "utf8");
  let changed = 0;
  for (const page of pages(dist)) {
    const before = readFileSync(page, "utf8");
    const after = addPreloads(before, read);
    if (after !== before) {
      writeFileSync(page, after);
      changed += 1;
    }
  }
  console.log(`modulepreload added to ${changed} pages`);
}
