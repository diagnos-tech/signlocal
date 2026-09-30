import { describe, expect, it } from "vitest";
import * as sdk from "../src/index";

// Contract smoke test: the public surface exists. Behavior tests are written
// by the SDK track from SPEC.md.
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
