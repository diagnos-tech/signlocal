import { describe, expect, test } from "bun:test";
import { addPreloads, dependencies } from "../../scripts/preload-modules.ts";

const files: Record<string, string> = {
  "/_astro/page.js": 'import{n as e}from"./strings.js";import{t}from"./tabs.js";var a=1;',
  "/_astro/strings.js": "export{x as n};",
  "/_astro/tabs.js":
    'import{n as e}from"./strings.js";import"./sdk.js";const lazy=()=>import("./later.js");',
  "/_astro/sdk.js": "export{};",
  "/_astro/later.js": 'import"./never.js";',
};
const read = (path: string) => files[path] ?? "";

describe("dependencies", () => {
  test("follows static imports once, nearest first, and leaves dynamic imports alone", () => {
    expect(dependencies("/_astro/page.js", read)).toEqual([
      "/_astro/strings.js",
      "/_astro/tabs.js",
      "/_astro/sdk.js",
    ]);
  });

  test("a module without imports has none", () => {
    expect(dependencies("/_astro/strings.js", read)).toEqual([]);
  });
});

describe("addPreloads", () => {
  const page = (extra = "") =>
    `<head><link rel="stylesheet" href="/_astro/style.css">${extra}</head><body><script type="module" src="/_astro/page.js"></script></body>`;

  test("lists the dependencies right after the stylesheet, not the page script itself", () => {
    const html = addPreloads(page(), read);
    expect(html).toContain(
      '<link rel="stylesheet" href="/_astro/style.css"><link rel="modulepreload" href="/_astro/strings.js"><link rel="modulepreload" href="/_astro/tabs.js"><link rel="modulepreload" href="/_astro/sdk.js"></head>',
    );
    expect(html).not.toContain('modulepreload" href="/_astro/page.js"');
  });

  test("running twice changes nothing", () => {
    const once = addPreloads(page(), read);
    expect(addPreloads(once, read)).toBe(once);
  });

  test("a page without module scripts is left as it is", () => {
    const html = '<head><link rel="stylesheet" href="/_astro/style.css"></head><body></body>';
    expect(addPreloads(html, read)).toBe(html);
  });
});
