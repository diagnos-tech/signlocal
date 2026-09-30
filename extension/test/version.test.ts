import { describe, expect, it } from "vitest";
import { isOlder } from "../src/shared/version";

// Contract smoke test; behavior tests come from SPEC.md §5.
describe("isOlder", () => {
  it("is a function", () => {
    expect(typeof isOlder).toBe("function");
  });
});
