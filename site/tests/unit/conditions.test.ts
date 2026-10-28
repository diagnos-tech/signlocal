import { describe, expect, test } from "bun:test";
import { type Conditions, type Facts, matches } from "../../src/wizard/conditions.ts";

const cases: ReadonlyArray<readonly [string, Conditions, Facts, boolean]> = [
  ["no conditions always match", {}, { os: "windows" }, true],
  ["a single value", { os: "windows" }, { os: "windows" }, true],
  ["a different value", { os: "windows" }, { os: "macos" }, false],
  ["a list matches any of its values", { os: "windows macos" }, { os: "macos" }, true],
  ["a list rejects others", { os: "windows macos" }, { os: "linux" }, false],
  ["negation excludes the listed value", { browser: "!safari" }, { browser: "safari" }, false],
  ["negation allows the others", { browser: "!safari" }, { browser: "chrome" }, true],
  ["negation of a list", { browser: "!safari firefox" }, { browser: "firefox" }, false],
  ["negation passes when the fact is unset", { browser: "!safari" }, {}, true],
  ["a positive list fails when the fact is unset", { os: "linux" }, {}, false],
  [
    "all keys must match",
    { os: "linux", distro: "deb rpm" },
    { os: "linux", distro: "other" },
    false,
  ],
  [
    "all keys matching",
    { os: "linux", distro: "deb rpm", release: "yes" },
    { os: "linux", distro: "rpm", release: "yes" },
    true,
  ],
  ["extra whitespace is ignored", { arch: "  x64   arm64 " }, { arch: "arm64" }, true],
  ["null attributes are absent", { os: null }, { os: "windows" }, true],
  [
    "channel and release",
    { channel: "dev", release: "no" },
    { channel: "dev", release: "no" },
    true,
  ],
];

describe("matches", () => {
  for (const [name, conditions, facts, expected] of cases) {
    test(name, () => expect(matches(conditions, facts)).toBe(expected));
  }
});
