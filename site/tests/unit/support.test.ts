import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { linuxTested, readSupport } from "../../scripts/data/support.ts";

const repo = new URL("../../..", import.meta.url).pathname.replace(/\/$/, "");

describe("readSupport (against the real repository files)", () => {
  const support = readSupport(repo);

  test("minimum versions come from the extension manifest and Info.plist.in", () => {
    expect(support.chromium).toMatch(/^\d+$/);
    expect(support.firefox).toMatch(/^\d+$/);
    expect(support.macos).toMatch(/^\d+$/);
  });

  test("the Linux distributions come from e2e.yml", () => {
    expect(support.linuxTested.length).toBeGreaterThanOrEqual(3);
    expect(support.linuxTested.some((name) => name.startsWith("Ubuntu"))).toBe(true);
  });
});

describe("linuxTested", () => {
  test("reads the Ubuntu runners and the container matrix", () => {
    const workflow = [
      "jobs:",
      "  runner:",
      "    strategy:",
      "      matrix:",
      "        os: " +
        [
          "$",
          '{{ fromJSON(cond && \'["windows-latest","ubuntu-22.04","ubuntu-24.04"]\' || \'["ubuntu-24.04"]\') }}',
        ].join(""),
      "  container:",
      "    strategy:",
      "      matrix:",
      "        include:",
      '          - { image: "debian:12", os: debian-12 }',
      '          - { image: "fedora:42", os: fedora-42 }',
    ].join("\n");
    expect(linuxTested(workflow)).toEqual([
      "Ubuntu 22.04",
      "Ubuntu 24.04",
      "Debian 12",
      "Fedora 42",
    ]);
  });

  test("an unknown distribution fails loudly", () => {
    const workflow =
      "jobs:\n  c:\n    strategy:\n      matrix:\n        include:\n          - { os: gentoo-1 }\n";
    expect(() => linuxTested(workflow)).toThrow("gentoo-1");
  });

  test("a workflow without Linux fails", () => {
    expect(() => linuxTested("jobs:\n  a:\n    runs-on: x\n")).toThrow("no Linux distributions");
  });
});

describe("manifest and plist formats", () => {
  test("the sources still have the lines the extractors read", () => {
    expect(readFileSync(`${repo}/extension/build/manifest.ts`, "utf8")).toMatch(
      /minimum_chrome_version:\s*"\d+"/,
    );
    expect(readFileSync(`${repo}/packaging/macos/Info.plist.in`, "utf8")).toMatch(
      /LSMinimumSystemVersion<\/key>\s*<string>\d+/,
    );
  });
});
