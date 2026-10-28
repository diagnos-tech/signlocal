import { describe, expect, it } from "vitest";
import { HINTS } from "../src/hints";
import { isSignlocalError, SignlocalError } from "../src/index";

describe("SignlocalError", () => {
  it("gives every protocol code a hint and the docs link", () => {
    for (const code of Object.keys(HINTS) as Array<keyof typeof HINTS>) {
      const error = new SignlocalError(code, "m");
      expect(error.hint.length).toBeGreaterThan(20);
      expect(error.docsUrl).toBe(`https://signlocal.diagnos.health/developers.html#error-${code}`);
      expect(error.code).toBe(code);
    }
  });

  it("keeps message, name and details as before", () => {
    const error = new SignlocalError("ClientOutdated", "old", { installed: "1", required: "2" });
    expect(error.message).toBe("old");
    expect(error.name).toBe("SignlocalError");
    expect(error.details).toEqual({ installed: "1", required: "2" });
  });

  it("narrows with isSignlocalError", () => {
    const error: unknown = new SignlocalError("UserCancelled", "m");
    expect(isSignlocalError(error)).toBe(true);
    expect(isSignlocalError(error, "UserCancelled", "Aborted")).toBe(true);
    expect(isSignlocalError(error, "Busy")).toBe(false);
    expect(isSignlocalError(new Error("x"))).toBe(false);
  });
});
