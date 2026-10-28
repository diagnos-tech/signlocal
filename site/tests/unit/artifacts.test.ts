import { describe, expect, test } from "bun:test";
import { readFileSync } from "node:fs";
import { appFile, artifactNames, requiredFiles } from "../../src/platform/artifacts.ts";

const release = readFileSync(
  new URL("../../../.github/workflows/release.yml", import.meta.url),
  "utf8",
);

describe("artifactNames", () => {
  const names = artifactNames("signlocal", "0.2.0");

  test("names follow the packaging contract", () => {
    expect(names.windows.x64).toBe("signlocal-0.2.0-windows-x64.zip");
    expect(names.macos).toBe("signlocal-0.2.0-macos-universal.zip");
    expect(names.deb.arm64).toBe("signlocal_0.2.0_arm64.deb");
    expect(names.rpm.x64).toBe("signlocal-0.2.0-1.x86_64.rpm");
    expect(names.tarball.arm64).toBe("signlocal-0.2.0-linux-arm64.tar.gz");
    expect(names.extensionFirefox).toBe("signlocal-extension-0.2.0-firefox.zip");
  });

  test("the manual download of the app depends on system, processor and distribution", () => {
    expect(appFile(names, "windows", "arm64", "deb")).toBe("signlocal-0.2.0-windows-arm64.zip");
    expect(appFile(names, "macos", "x64", "deb")).toBe("signlocal-0.2.0-macos-universal.zip");
    expect(appFile(names, "linux", "x64", "deb")).toBe("signlocal_0.2.0_amd64.deb");
    expect(appFile(names, "linux", "arm64", "rpm")).toBe("signlocal-0.2.0-1.aarch64.rpm");
    expect(appFile(names, "linux", "x64", "other")).toBeNull();
  });
});

/** The workflow writes the version as ${v} or $v in shell. */
const BRACED_VERSION = ["$", "{v}"].join("");

describe("drift guard against .github/workflows/release.yml", () => {
  /** The `expected=` list of the publish job, with $v and ${v} replaced by the version. */
  function publishedFiles(version: string): string[] {
    const block = /expected="([^"]+)"/.exec(release)?.[1];
    if (!block) throw new Error('release.yml has no expected="…" list');
    return block
      .split(/\s+/)
      .filter(Boolean)
      .map((name) => name.replaceAll(BRACED_VERSION, version).replaceAll("$v", version));
  }

  test("the site knows exactly the files the release workflow publishes (plus the installers)", () => {
    const version = "9.8.7";
    const known = requiredFiles("signlocal", version);
    const published = [...publishedFiles(version), "install.sh", "install.ps1"];
    expect([...known].sort()).toEqual([...published].sort());
  });
});
