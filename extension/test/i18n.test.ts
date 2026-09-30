import { readdirSync, readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";

const dir = new URL("../../i18n/", import.meta.url);
const locales = readdirSync(dir)
  .filter((name) => name.endsWith(".toml"))
  .map((name) => name.replace(/\.toml$/, ""));

/** `[popup]` entries of a locale as key -> value (single-line strings only). */
function popupSection(locale: string): Map<string, string> {
  const text = readFileSync(new URL(`${locale}.toml`, dir), "utf8");
  const entries = new Map<string, string>();
  let inPopup = false;
  for (const line of text.split("\n")) {
    const header = /^\[([^\]]+)\]\s*$/.exec(line);
    if (header) inPopup = header[1] === "popup";
    else if (inPopup) {
      const entry = /^([a-z_]+)\s*=\s*"(.*)"\s*$/.exec(line);
      if (entry?.[1] !== undefined && entry[2] !== undefined) entries.set(entry[1], entry[2]);
    }
  }
  return entries;
}

const placeholders = (value: string): string[] =>
  [...value.matchAll(/\{(\w+)\}/g)].map((m) => m[1] ?? "").sort();

/** Keys each popup state needs (docs/ux.md §9), so a state can never render an empty string. */
const keysByState: Record<string, string[]> = {
  checking: ["checking"],
  ready: ["ready_title", "ready_body", "open_diagnostics"],
  missing: ["missing_title", "missing_body", "download", "activate"],
  outdated: ["outdated_title", "outdated_body", "update_in"],
  error: ["error_title", "error_body", "retry", "redownload"],
  unsupported: ["unsupported_title", "unsupported_body"],
};

describe("popup i18n catalog", () => {
  const en = popupSection("en");

  it("finds the seven locales", () => {
    expect(locales.sort()).toEqual(["de", "en", "es", "fr", "it", "pt-BR", "pt-PT"]);
  });

  it.each(Object.entries(keysByState))("en defines every key of the %s state", (_state, keys) => {
    for (const key of keys) expect(en.get(key), key).toBeTruthy();
  });

  it("has the footer and privacy keys", () => {
    expect(en.get("footer_versions")).toContain("{ext}");
    expect(en.get("footer_versions")).toContain("{app}");
    expect(en.get("privacy")).toBeTruthy();
  });

  it("uses the placeholders the view fills", () => {
    expect(placeholders(en.get("ready_body") ?? "")).toEqual(["version"]);
    expect(placeholders(en.get("outdated_body") ?? "")).toEqual(["installed", "required"]);
    expect(placeholders(en.get("download") ?? "")).toEqual(["os"]);
    expect(placeholders(en.get("update_in") ?? "")).toEqual(["store"]);
  });

  it.each(locales)("%s has exactly the keys and placeholders of en", (locale) => {
    const other = popupSection(locale);
    expect([...other.keys()].sort()).toEqual([...en.keys()].sort());
    for (const [key, value] of en)
      expect(placeholders(other.get(key) ?? ""), key).toEqual(placeholders(value));
  });

  it("has no plural tables in the popup (chrome.i18n has none)", () => {
    const text = readFileSync(new URL("en.toml", dir), "utf8");
    expect(text).not.toMatch(/^\[popup\.[a-z_]+\]/m);
  });
});
