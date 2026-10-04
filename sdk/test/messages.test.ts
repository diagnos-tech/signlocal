import { describe, expect, it, vi } from "vitest";
import { WebSignError } from "../src/errors";
import { errorText } from "../src/messages";
import { ERROR_CODES } from "./helpers/fixtures";

const SILENT = ["InvalidRequest", "PinIncorrect", "ClientOutdated"];

describe("errorText()", () => {
  it("returns the English title and body", () => {
    expect(errorText("ExtensionMissing", "en")).toEqual({
      title: "Install the SignLocal extension",
      body: "To sign in this browser, install the free extension.",
    });
  });

  for (const code of ERROR_CODES) {
    if (SILENT.includes(code)) {
      it(`${code} has no user text (site bug or internal)`, () => {
        for (const locale of ["en", "pt-BR", "de", "xx"])
          expect(errorText(code, locale)).toBeUndefined();
      });
    } else {
      it(`${code} has a non-empty title and body in every locale`, () => {
        for (const locale of ["en", "pt-BR", "pt-PT", "es", "fr", "it", "de"]) {
          const text = errorText(code, locale);
          expect(text?.title.length).toBeGreaterThan(0);
          expect(text?.body.length).toBeGreaterThan(0);
        }
      });
    }
  }

  it("pt-BR is Portuguese", () => {
    expect(errorText("ExtensionMissing", "pt-BR")?.title).toBe("Instale a extensão SignLocal");
  });

  const closest: [string, string][] = [
    ["pt", "pt-BR"],
    ["pt-br", "pt-BR"],
    ["es-MX", "es"],
    ["es-419", "es"],
    ["fr-CA", "fr"],
    ["de-AT", "de"],
    ["it-CH", "it"],
    ["pt-PT", "pt-PT"],
    ["en-GB", "en"],
  ];
  for (const [asked, expected] of closest) {
    it(`${asked} resolves to ${expected}`, () => {
      expect(errorText("Timeout", asked)).toEqual(errorText("Timeout", expected));
    });
  }

  for (const asked of ["", "xx", "ja-JP", "zh-Hans", "not a locale"]) {
    it(`unknown locale ${JSON.stringify(asked)} falls back to English`, () => {
      expect(errorText("Timeout", asked)).toEqual(errorText("Timeout", "en"));
    });
  }

  it("locales are distinct texts", () => {
    expect(errorText("Timeout", "de")).not.toEqual(errorText("Timeout", "en"));
  });
});

describe("errorText() with an error", () => {
  it("fills {installed} and {required} from details", () => {
    const error = new WebSignError("AppOutdated", "old", { installed: "1.0.0", required: "1.2.0" });
    const text = errorText(error, "en");
    expect(text?.body).toContain("1.0.0");
    expect(text?.body).toContain("1.2.0");
    expect(text?.body).not.toMatch(/\{(installed|required)\}/);
  });

  it("leaves placeholders it has no value for", () => {
    expect(errorText("AppOutdated", "en")?.body).toMatch(/\{installed\}/);
    expect(errorText({ code: "AppOutdated", details: { required: "2" } }, "en")?.body).toMatch(
      /\{installed\}/,
    );
  });

  it("returns undefined for undefined, so status().problem can be passed as is", () => {
    expect(errorText(undefined, "en")).toBeUndefined();
  });

  it("defaults to the browser's language", () => {
    vi.stubGlobal("navigator", { language: "de-DE" });
    expect(errorText("Timeout")).toEqual(errorText("Timeout", "de"));
    vi.unstubAllGlobals();
  });

  it("defaults to English without a navigator", () => {
    vi.stubGlobal("navigator", undefined);
    expect(errorText("Timeout")).toEqual(errorText("Timeout", "en"));
    vi.unstubAllGlobals();
  });
});
