import { describe, expect, it } from "vitest";
import { HINTS } from "../src/hints";
import { isWebSignError, WebSignError } from "../src/index";

describe("WebSignError", () => {
  it("gives every protocol code a hint and the docs link", () => {
    for (const code of Object.keys(HINTS) as Array<keyof typeof HINTS>) {
      const error = new WebSignError(code, "m");
      expect(error.hint.length).toBeGreaterThan(20);
      expect(error.docsUrl).toBe(
        `https://diagnos-tech.github.io/signlocal/developers.html#error-${code}`,
      );
      expect(error.code).toBe(code);
    }
  });

  it("keeps message, name and details as before", () => {
    const error = new WebSignError("ClientOutdated", "old", { installed: "1", required: "2" });
    expect(error.message).toBe("old");
    expect(error.name).toBe("WebSignError");
    expect(error.details).toEqual({ installed: "1", required: "2" });
  });

  it("narrows with isWebSignError", () => {
    const error: unknown = new WebSignError("UserCancelled", "m");
    expect(isWebSignError(error)).toBe(true);
    expect(isWebSignError(error, "UserCancelled", "Aborted")).toBe(true);
    expect(isWebSignError(error, "Busy")).toBe(false);
    expect(isWebSignError(new Error("x"))).toBe(false);
  });
});
