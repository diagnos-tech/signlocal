import { readFileSync } from "node:fs";
import { join } from "node:path";
import { expect, it } from "vitest";
import { VERSION } from "../src/version";

it("reports the version of package.json in hello", () => {
  const manifest = JSON.parse(
    readFileSync(join(import.meta.dirname, "..", "package.json"), "utf8"),
  );
  expect(VERSION).toBe(manifest.version);
});
