import { describe, expect, test } from "bun:test";
import { browserValidFor } from "../../src/platform/catalog.ts";
import { BROWSER_IDS, CERT_IDS, OS_IDS } from "../../src/platform/types.ts";
import { recipe } from "../../src/wizard/recipe.ts";

describe("recipe", () => {
  test("every browser but Safari: install, extension, connect, certificate, test", () => {
    expect(recipe({ browser: "chrome" }, true)).toEqual([
      "install",
      "extension",
      "connect",
      "certificate",
      "test",
    ]);
  });

  test("Safari opens the app once before it turns the extension on", () => {
    expect(recipe({ browser: "safari" }, true)).toEqual([
      "install",
      "connect",
      "extension",
      "certificate",
      "test",
    ]);
  });

  test("with no release there is nothing to install", () => {
    expect(recipe({ browser: "chrome" }, false)).toEqual(["no-release", "certificate"]);
  });

  test("every valid combination produces a recipe that starts with a card to act on", () => {
    for (const os of OS_IDS) {
      for (const browser of BROWSER_IDS.filter((candidate) => browserValidFor(candidate, os))) {
        for (const cert of CERT_IDS) {
          const cards = recipe({ os, browser, cert }, true);
          expect(cards[0]).toBe("install");
          expect(cards.at(-1)).toBe("test");
          expect(new Set(cards).size).toBe(cards.length);
        }
      }
    }
  });
});
