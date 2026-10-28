import { describe, expect, test } from "bun:test";
import { choose, matchTag } from "../../src/i18n/choose.ts";

describe("matchTag", () => {
  const cases: ReadonlyArray<readonly [string, string | undefined]> = [
    ["en", "en"],
    ["en-US", "en"],
    ["pt", "pt-BR"],
    ["pt-BR", "pt-BR"],
    ["pt_br", "pt-BR"],
    ["pt-PT", "pt-PT"],
    ["PT-pt", "pt-PT"],
    ["es-MX", "es"],
    ["de-AT", "de"],
    ["fr-CA", "fr"],
    ["it", "it"],
    ["zh-CN", undefined],
    ["", undefined],
  ];
  for (const [tag, expected] of cases) {
    test(`${JSON.stringify(tag)} → ${expected}`, () =>
      expect(matchTag(tag)).toBe(expected as never));
  }
  test("null and undefined match nothing", () => {
    expect(matchTag(null)).toBeUndefined();
    expect(matchTag(undefined)).toBeUndefined();
  });
});

describe("choose", () => {
  test("?lang wins over everything", () => {
    expect(choose({ query: "de", saved: "fr", languages: ["pt-BR"] })).toBe("de");
  });
  test("the saved choice wins over the browser", () => {
    expect(choose({ saved: "en", languages: ["pt-BR"] })).toBe("en");
  });
  test("the first browser language the site has wins", () => {
    expect(choose({ languages: ["zh-CN", "es-AR", "de"] })).toBe("es");
  });
  test("English when nothing matches", () => {
    expect(choose({ languages: ["zh", "ja"] })).toBe("en");
    expect(choose({})).toBe("en");
  });
  test("an invalid ?lang falls through to the saved choice", () => {
    expect(choose({ query: "xx", saved: "it" })).toBe("it");
  });
});
