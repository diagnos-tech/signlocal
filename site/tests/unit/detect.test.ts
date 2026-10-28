import { describe, expect, test } from "bun:test";
import { archFromHints, type DetectionInput, detect } from "../../src/platform/detect.ts";
import type { Detection } from "../../src/platform/types.ts";
import fixtures from "../fixtures/user-agents.json";

interface Fixture {
  readonly name: string;
  readonly input: DetectionInput;
  readonly expect: Detection;
}

describe("detect", () => {
  for (const fixture of fixtures as Fixture[]) {
    test(fixture.name, () => {
      expect(detect(fixture.input)).toEqual(fixture.expect);
    });
  }

  test("the fixtures cover every platform and browser the wizard offers", () => {
    const results = (fixtures as Fixture[]).map((fixture) => detect(fixture.input));
    for (const platform of ["windows", "macos", "linux", "mobile", "chromeos", null]) {
      expect(results.some((result) => result.platform === platform)).toBe(true);
    }
    for (const browser of ["chrome", "edge", "firefox", "opera", "brave", "safari", "chromium"]) {
      expect(results.some((result) => result.browser === browser)).toBe(true);
    }
  });
});

describe("archFromHints", () => {
  test("an ARM processor is arm64", () => {
    expect(archFromHints({ architecture: "arm", bitness: "64" })).toBe("arm64");
  });
  test("x86 with 64 bits is x64", () => {
    expect(archFromHints({ architecture: "x86", bitness: "64" })).toBe("x64");
  });
  test("32-bit x86 and missing hints say nothing", () => {
    expect(archFromHints({ architecture: "x86", bitness: "32" })).toBeNull();
    expect(archFromHints({})).toBeNull();
  });
});
