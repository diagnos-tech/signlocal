import { describe, expect, it } from "vitest";
import { fromBrands, fromUserAgent, majorMinor } from "../src/shared/browser-name";

const brands = (...names: [string, string][]) =>
  names.map(([brand, version]) => ({ brand, version }));

describe("fromBrands (UA-CH)", () => {
  it.each([
    [brands(["Not)A;Brand", "99"], ["Google Chrome", "129"], ["Chromium", "129"]), "chrome"],
    [brands(["Microsoft Edge", "128"], ["Chromium", "128"]), "edge"],
    [brands(["Brave", "129"], ["Chromium", "129"]), "brave"],
    [brands(["Opera", "114"], ["Chromium", "128"]), "opera"],
    [brands(["Vivaldi", "6.9"], ["Chromium", "128"]), "vivaldi"],
    [brands(["Not)A;Brand", "99"], ["Chromium", "129"]), "chromium"],
    [[], "chromium"],
  ])("%j → %s", (list, name) => {
    expect(fromBrands(list).name).toBe(name);
  });

  it("prefers the specific brand's version over Chromium's", () => {
    expect(fromBrands(brands(["Opera", "114.0.5282"], ["Chromium", "128"]))).toEqual({
      name: "opera",
      version: "114.0",
    });
  });
});

describe("fromUserAgent", () => {
  it.each([
    ["Mozilla/5.0 (X11; Linux x86_64; rv:131.0) Gecko/20100101 Firefox/131.0", "firefox", "131.0"],
    [
      "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/605.1.15 (KHTML, like Gecko) Version/18.0 Safari/605.1.15",
      "safari",
      "18.0",
    ],
    [
      "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/129.0.0.0 Safari/537.36",
      "other",
      "0.0",
    ],
    ["Node.js/22", "other", "0.0"],
  ])("%s → %s %s", (ua, name, version) => {
    expect(fromUserAgent(ua)).toEqual({ name, version });
  });
});

describe("majorMinor", () => {
  it.each([
    ["129.0.6668.58", "129.0"],
    ["129", "129.0"],
    ["", ".0"],
  ])("%j → %j", (input, output) => {
    expect(majorMinor(input)).toBe(output);
  });
});
