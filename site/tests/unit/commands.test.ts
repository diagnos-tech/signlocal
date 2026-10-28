import { describe, expect, test } from "bun:test";
import {
  linuxAutomatic,
  linuxPackage,
  macosAutomatic,
  verifyCommand,
  windowsAutomatic,
} from "../../src/platform/commands.ts";

const sha = "3f2a9c1e".repeat(8);
const installer = {
  url: "https://github.com/o/r/releases/download/v1.2.3/install.sh",
  sha256: sha,
};

describe("commands", () => {
  test("Windows: one script block that checks the installer's hash in upper case and stops on a mismatch", () => {
    const command = windowsAutomatic(
      { url: "https://github.com/o/r/releases/download/v1.2.3/install.ps1", sha256: sha },
      "1.2.3",
    );
    expect(command).toBe(
      [
        "& {",
        "  $ErrorActionPreference = 'Stop'",
        "  $ProgressPreference = 'SilentlyContinue'",
        "  [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12",
        "  $installer = Join-Path $env:TEMP 'signlocal-install.ps1'",
        "  Invoke-WebRequest -UseBasicParsing -Uri 'https://github.com/o/r/releases/download/v1.2.3/install.ps1' -OutFile $installer",
        `  if ((Get-FileHash -Algorithm SHA256 -LiteralPath $installer).Hash -ne '${sha.toUpperCase()}') {`,
        "    Remove-Item -LiteralPath $installer",
        "    throw 'The download does not match its checksum. Nothing was installed.'",
        "  }",
        "  powershell -NoProfile -ExecutionPolicy Bypass -File $installer -Version 1.2.3",
        "}",
      ].join("\n"),
    );
  });

  test("macOS: download, check the hash with shasum, run", () => {
    expect(macosAutomatic(installer, "1.2.3")).toBe(
      `cd "$(mktemp -d)" && curl -fsSLO '${installer.url}' && echo '${sha}  install.sh' | shasum -a 256 -c && sh install.sh --version 1.2.3`,
    );
  });

  test("Linux: --system for deb and rpm, --yes for the per-user install", () => {
    const base = `cd "$(mktemp -d)" && curl -fsSLO '${installer.url}' && echo '${sha}  install.sh' | sha256sum -c && sh install.sh --version 1.2.3`;
    expect(linuxAutomatic(installer, "1.2.3", "system")).toBe(`${base} --system`);
    expect(linuxAutomatic(installer, "1.2.3", "user")).toBe(`${base} --yes`);
  });

  test("Linux packages: check the file in the Downloads folder, then install it", () => {
    const file = { name: "signlocal_1.2.3_amd64.deb", url: "https://example.test/x", sha256: sha };
    expect(linuxPackage(file, "deb")).toBe(
      `cd "$(xdg-user-dir DOWNLOAD 2>/dev/null || echo ~/Downloads)" && echo '${sha}  signlocal_1.2.3_amd64.deb' | sha256sum -c && sudo apt install './signlocal_1.2.3_amd64.deb'`,
    );
    expect(linuxPackage({ ...file, name: "signlocal-1.2.3-1.x86_64.rpm" }, "rpm")).toContain(
      "sudo dnf install './signlocal-1.2.3-1.x86_64.rpm'",
    );
  });

  test("verify commands per system, with the hash in the case each tool prints", () => {
    expect(verifyCommand("windows", "a.zip", sha)).toBe(
      `(Get-FileHash -Algorithm SHA256 -LiteralPath "$HOME\\Downloads\\a.zip").Hash -eq '${sha.toUpperCase()}'`,
    );
    expect(verifyCommand("macos", "a.zip", sha)).toBe(
      `cd ~/Downloads && echo '${sha}  a.zip' | shasum -a 256 -c`,
    );
    expect(verifyCommand("linux", "a.zip", sha)).toBe(
      `cd "$(xdg-user-dir DOWNLOAD 2>/dev/null || echo ~/Downloads)" && echo '${sha}  a.zip' | sha256sum -c`,
    );
  });

  test("hashes are written in lower case on Unix and upper case on Windows whatever the input", () => {
    const mixed = { ...installer, sha256: sha.toLowerCase() };
    expect(macosAutomatic(mixed, "1.0.0")).toContain(sha.toLowerCase());
    expect(windowsAutomatic(mixed, "1.0.0")).toContain(sha.toUpperCase());
  });
});
