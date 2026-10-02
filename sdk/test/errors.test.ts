import { describe, expect, it } from "vitest";
import { isWebSignError, knownCode, WebSignError } from "../src/errors";
import { HOMEPAGE } from "../src/project";
import { ERROR_CODES } from "./helpers/fixtures";

describe("WebSignError", () => {
  for (const code of ERROR_CODES) {
    it(`${code} carries a hint and a stable docs anchor`, () => {
      const error = new WebSignError(code, "what happened");
      expect(error.hint.length).toBeGreaterThan(10);
      expect(error.hint).not.toBe(error.message);
      expect(error.docsUrl).toBe(`${HOMEPAGE}developers.html#error-${code}`);
    });
  }

  it("is an Error named WebSignError with code, message and details", () => {
    const error = new WebSignError("AppOutdated", "old app", { installed: "1.0", required: "1.2" });
    expect(error).toBeInstanceOf(Error);
    expect(error.name).toBe("WebSignError");
    expect(error.message).toBe("old app");
    expect(error.details).toEqual({ installed: "1.0", required: "1.2" });
  });

  it("hints never repeat each other: each code says its own next step", () => {
    const hints = ERROR_CODES.map((code) => new WebSignError(code, "").hint);
    expect(new Set(hints).size).toBe(hints.length);
  });

  it("an unknown wire code maps to Internal", () => {
    expect(knownCode("SomethingNew")).toBe("Internal");
    expect(knownCode("toString")).toBe("Internal");
    expect(knownCode("Busy")).toBe("Busy");
  });
});

describe("isWebSignError()", () => {
  const cancelled = new WebSignError("UserCancelled", "closed");

  it("recognizes any WebSignError without codes", () => {
    expect(isWebSignError(cancelled)).toBe(true);
    expect(isWebSignError(new Error("x"))).toBe(false);
    expect(isWebSignError({ name: "WebSignError", code: "UserCancelled" })).toBe(false);
    expect(isWebSignError(undefined)).toBe(false);
  });

  it("filters by code", () => {
    expect(isWebSignError(cancelled, "UserCancelled", "Aborted")).toBe(true);
    expect(isWebSignError(cancelled, "Timeout")).toBe(false);
  });
});
