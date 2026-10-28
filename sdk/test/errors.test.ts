import { describe, expect, it } from "vitest";
import { isSignlocalError, knownCode, SignlocalError } from "../src/errors";
import { HOMEPAGE } from "../src/project";
import { ERROR_CODES } from "./helpers/fixtures";

describe("SignlocalError", () => {
  for (const code of ERROR_CODES) {
    it(`${code} carries a hint and a stable docs anchor`, () => {
      const error = new SignlocalError(code, "what happened");
      expect(error.hint.length).toBeGreaterThan(10);
      expect(error.hint).not.toBe(error.message);
      expect(error.docsUrl).toBe(`${HOMEPAGE}developers.html#error-${code}`);
    });
  }

  it("is an Error named SignlocalError with code, message and details", () => {
    const error = new SignlocalError("AppOutdated", "old app", {
      installed: "1.0",
      required: "1.2",
    });
    expect(error).toBeInstanceOf(Error);
    expect(error.name).toBe("SignlocalError");
    expect(error.message).toBe("old app");
    expect(error.details).toEqual({ installed: "1.0", required: "1.2" });
  });

  it("hints never repeat each other: each code says its own next step", () => {
    const hints = ERROR_CODES.map((code) => new SignlocalError(code, "").hint);
    expect(new Set(hints).size).toBe(hints.length);
  });

  it("an unknown wire code maps to Internal", () => {
    expect(knownCode("SomethingNew")).toBe("Internal");
    expect(knownCode("toString")).toBe("Internal");
    expect(knownCode("Busy")).toBe("Busy");
  });
});

describe("isSignlocalError()", () => {
  const cancelled = new SignlocalError("UserCancelled", "closed");

  it("recognizes any SignlocalError without codes", () => {
    expect(isSignlocalError(cancelled)).toBe(true);
    expect(isSignlocalError(new Error("x"))).toBe(false);
    expect(isSignlocalError({ name: "SignlocalError", code: "UserCancelled" })).toBe(false);
    expect(isSignlocalError(undefined)).toBe(false);
  });

  it("filters by code", () => {
    expect(isSignlocalError(cancelled, "UserCancelled", "Aborted")).toBe(true);
    expect(isSignlocalError(cancelled, "Timeout")).toBe(false);
  });
});
