import { describe, expect, it } from "vitest";
import { isOlder } from "../src/shared/version";

describe("isOlder", () => {
  it.each([
    ["0.9.9", "1.0.0", true],
    ["1.0", "1.0.0", false],
    ["1.10.0", "1.9.9", false],
    ["1.0.0-beta", "1.0.0", true],
    ["", "0.0.1", true],
  ])("(%j, %j) -> %s (SPEC §5 vectors)", (version, minimum, expected) => {
    expect(isOlder(version, minimum)).toBe(expected);
  });

  it("is false for equal versions and for a missing part equal to zero", () => {
    expect(isOlder("1.2.3", "1.2.3")).toBe(false);
    expect(isOlder("1", "1.0.0")).toBe(false);
    expect(isOlder("1.0.0", "1")).toBe(false);
  });

  it("compares numerically, not lexically, part by part", () => {
    expect(isOlder("1.9.0", "1.10.0")).toBe(true);
    expect(isOlder("2.0.0", "1.99.99")).toBe(false);
    expect(isOlder("1.2", "1.2.1")).toBe(true);
  });

  it.each(["abc", "1.x.0", "v1.0.0", "1..0", "1.0.0.0.0", " 1.0.0", "-1.0.0", "1.0.0+build"])(
    "treats non-numeric or malformed %j as older",
    (version) => {
      expect(isOlder(version, "0.0.1")).toBe(true);
    },
  );
});
