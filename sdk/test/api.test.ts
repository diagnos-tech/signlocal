import { describe, expect, it } from "vitest";
import * as sdk from "../src/index";

// Smoke test: the documented public surface exists (the exact key set is in security.test.ts).
describe("public API", () => {
  it("exports every documented function", () => {
    for (const name of [
      "status",
      "certificates",
      "sign",
      "installUrl",
      "onChange",
      "fingerprint",
    ]) {
      expect(typeof (sdk as Record<string, unknown>)[name]).toBe("function");
    }
    expect(typeof sdk.WebSignError).toBe("function");
  });
});
