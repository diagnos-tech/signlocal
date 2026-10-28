import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { GLIBC_MIN, SAFARI_MIN, WINDOWS_FLOOR } from "../../src/platform/facts.ts";

const document = readFileSync(
  new URL("../../../docs/architecture/compatibility.md", import.meta.url),
  "utf8",
);

describe("facts without a code source", () => {
  test("docs/architecture/compatibility.md still states them", () => {
    expect(document).toContain(WINDOWS_FLOOR);
    expect(document).toContain(`Safari ${SAFARI_MIN}+`);
    expect(document).toContain(`glibc ≥ ${GLIBC_MIN}`);
  });
});
