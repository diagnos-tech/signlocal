// Run: bun test site/tests   (or: node --test site/tests)
//
// i18n.js swaps text by key and silently keeps English when a key is missing,
// so a forgotten translation would never show up as an error in the browser.
import assert from "node:assert/strict";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { test } from "node:test";

const site = new URL("..", import.meta.url).pathname;
const read = (path) => readFileSync(join(site, path), "utf8");

function htmlFiles(dir = "") {
  return readdirSync(join(site, dir)).flatMap((name) => {
    const path = join(dir, name);
    if (statSync(join(site, path)).isDirectory()) return name === "tests" ? [] : htmlFiles(path);
    return name.endsWith(".html") ? [path] : [];
  });
}

const used = new Set(
  htmlFiles().flatMap((path) => [...read(path).matchAll(/data-i18n(?:-label)?="([^"]+)"/g)].map((m) => m[1])),
);
const locales = readdirSync(join(site, "locales")).filter((name) => name.endsWith(".json"));

test("every locale has exactly the keys the pages use", () => {
  assert.equal(locales.length, 7);
  for (const name of locales) {
    const keys = Object.keys(JSON.parse(read(`locales/${name}`)));
    assert.deepEqual([...used].filter((k) => !keys.includes(k)), [], `${name}: missing keys`);
    assert.deepEqual(keys.filter((k) => !used.has(k)), [], `${name}: keys no page uses`);
  }
});

test("no translation is empty", () => {
  for (const name of locales) {
    const dict = JSON.parse(read(`locales/${name}`));
    for (const [key, text] of Object.entries(dict)) assert.ok(text.trim(), `${name}: ${key} is empty`);
  }
});
