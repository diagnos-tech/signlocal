import { describe, expect, test } from "bun:test";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { parse } from "smol-toml";
import { LOCALE_CODES } from "../../src/i18n/codes.ts";
import { flatten, placeholders } from "../../src/i18n/messages.ts";

const root = new URL("../../src/", import.meta.url).pathname;
const file = (code: string) => readFileSync(`${root}i18n/messages/${code}.toml`, "utf8");
const catalogs = Object.fromEntries(LOCALE_CODES.map((code) => [code, flatten(file(code), code)]));
const english = catalogs.en ?? {};

describe("locale parity", () => {
  test("there are seven locale files and they are the ones the site lists", () => {
    const files = readdirSync(`${root}i18n/messages`).filter((name) => name.endsWith(".toml"));
    expect(files.map((name) => name.replace(".toml", "")).sort()).toEqual([...LOCALE_CODES].sort());
  });

  for (const code of LOCALE_CODES) {
    describe(code, () => {
      const catalog = catalogs[code] ?? {};

      test("has exactly the keys of en.toml", () => {
        expect(Object.keys(catalog).sort()).toEqual(Object.keys(english).sort());
      });

      test("every message has the same placeholders as English", () => {
        const wrong = Object.entries(english).filter(
          ([key, text]) =>
            placeholders(catalog[key] ?? "")
              .sort()
              .join() !== placeholders(text).sort().join(),
        );
        expect(wrong.map(([key]) => key)).toEqual([]);
      });

      test("no message is empty", () => {
        expect(
          Object.entries(catalog)
            .filter(([, text]) => text.trim() === "")
            .map(([key]) => key),
        ).toEqual([]);
      });

      test("rich-text markers are balanced and the links point to the same pages", () => {
        const links = (text: string) =>
          [...text.matchAll(/\]\(([a-z-]+(?:#[a-z-]+)?)\)/g)].map((match) => match[1]);
        const broken = Object.entries(english).filter(([key, text]) => {
          const translated = catalog[key] ?? "";
          const unbalanced = ["**", "`", "__"].some(
            (marker) => translated.split(marker).length % 2 === 0,
          );
          return unbalanced || links(text).join() !== links(translated).join();
        });
        expect(broken.map(([key]) => key)).toEqual([]);
      });

      test("never names the product the old way and never says the brand in capitals", () => {
        const text = Object.values(catalog).join("\n");
        expect(text).not.toMatch(/web-?e?sign/i);
        expect(text).not.toMatch(/\bSignLocal\b|\bSIGNLOCAL\b|\bSignlocal\b/);
      });

      test("has no plural tables: every section is a flat table of text", () => {
        const sections = Object.values(parse(file(code))) as Record<string, unknown>[];
        for (const section of sections) {
          for (const value of Object.values(section)) expect(typeof value).toBe("string");
        }
      });
    });
  }
});

describe("keys against the code", () => {
  function sources(directory: string): string[] {
    return readdirSync(directory).flatMap((name) => {
      const path = join(directory, name);
      if (statSync(path).isDirectory()) {
        return name === "messages" || name === "generated" ? [] : sources(path);
      }
      return /\.(astro|ts)$/.test(name) ? [readFileSync(path, "utf8")] : [];
    });
  }
  const code = sources(root).join("\n");

  test("every key is used by the site, and written out in full so it can be found", () => {
    const unused = Object.keys(english).filter((key) => !code.includes(`"${key}"`));
    expect(unused).toEqual([]);
  });
});
