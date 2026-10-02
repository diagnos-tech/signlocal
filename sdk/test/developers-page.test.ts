import { describe, expect, it } from "vitest";
import page from "../../site/developers.html?raw";
import de from "../../site/locales/de.json?raw";
import en from "../../site/locales/en.json?raw";
import es from "../../site/locales/es.json?raw";
import fr from "../../site/locales/fr.json?raw";
import it_ from "../../site/locales/it.json?raw";
import ptBR from "../../site/locales/pt-BR.json?raw";
import ptPT from "../../site/locales/pt-PT.json?raw";
import { WebSignError } from "../src/errors";
import { ERROR_CODES } from "./helpers/fixtures";

// WebSignError.docsUrl (and the desktop clients' errors) link to site/developers.html#error-<Code>.
describe("site/developers.html, the page docsUrl points to", () => {
  for (const code of ERROR_CODES) {
    it(`has the anchor of ${code}`, () => {
      const anchor = new URL(new WebSignError(code, "").docsUrl).hash.slice(1);
      expect(anchor).toBe(`error-${code}`);
      expect(page).toContain(`id="${anchor}"`);
    });
  }

  const keys = [...page.matchAll(/data-i18n="([^"]+)"/g)].map((m) => m[1] ?? "");
  const locales = { en, es, "pt-PT": ptPT, "pt-BR": ptBR, fr, it: it_, de };
  for (const [locale, text] of Object.entries(locales)) {
    it(`every text of the page is translated in ${locale}`, () => {
      const dictionary = JSON.parse(text) as Record<string, string>;
      expect(keys.filter((key) => !dictionary[key])).toEqual([]);
    });
  }

  it("the English page text matches en.json, so the page reads the same without JavaScript", () => {
    const dictionary = JSON.parse(en) as Record<string, string>;
    const decode = (s: string) =>
      s
        .replace(/&quot;/g, '"')
        .replace(/&lt;/g, "<")
        .replace(/&gt;/g, ">")
        .replace(/&amp;/g, "&");
    for (const [, key = "", inner = ""] of page.matchAll(/data-i18n="([^"]+)"[^>]*>([^<]*)</g)) {
      if (key.startsWith("dev.")) expect(decode(inner)).toBe(dictionary[key]);
    }
  });
});
