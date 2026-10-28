// The commands the install page shows, built from the release data. They are not translated.
//
// The automatic install downloads the installer script, checks it against a hash this page
// pins (the second channel: the site's own build next to the release files) and stops on a
// mismatch. The installer then verifies every file it downloads against SHA256SUMS itself
// (scripts/install/README.md).

export interface PinnedFile {
  /** browser_download_url of the release asset. */
  readonly url: string;
  /** SHA-256 in lower-case hex. */
  readonly sha256: string;
}

/** PowerShell 5.1 and 7: one script block, so a failed check stops everything. */
export function windowsAutomatic(installer: PinnedFile, version: string): string {
  return [
    "& {",
    "  $ErrorActionPreference = 'Stop'",
    "  $ProgressPreference = 'SilentlyContinue'",
    "  [Net.ServicePointManager]::SecurityProtocol = [Net.ServicePointManager]::SecurityProtocol -bor [Net.SecurityProtocolType]::Tls12",
    "  $installer = Join-Path $env:TEMP 'signlocal-install.ps1'",
    `  Invoke-WebRequest -UseBasicParsing -Uri '${installer.url}' -OutFile $installer`,
    `  if ((Get-FileHash -Algorithm SHA256 -LiteralPath $installer).Hash -ne '${installer.sha256.toUpperCase()}') {`,
    "    Remove-Item -LiteralPath $installer",
    "    throw 'The download does not match its checksum. Nothing was installed.'",
    "  }",
    `  powershell -NoProfile -ExecutionPolicy Bypass -File $installer -Version ${version}`,
    "}",
  ].join("\n");
}

export const WINDOWS_REGISTER = ".\\signlocal.exe install";

export function macosAutomatic(installer: PinnedFile, version: string): string {
  return [
    `cd "$(mktemp -d)" && curl -fsSLO '${installer.url}'`,
    `echo '${installer.sha256}  install.sh' | shasum -a 256 -c`,
    `sh install.sh --version ${version}`,
  ].join(" && ");
}

/** `system`: the deb or rpm through sudo; `user`: per user, no root. */
export function linuxAutomatic(
  installer: PinnedFile,
  version: string,
  mode: "system" | "user",
): string {
  const flag = mode === "system" ? "--system" : "--yes";
  return [
    `cd "$(mktemp -d)" && curl -fsSLO '${installer.url}'`,
    `echo '${installer.sha256}  install.sh' | sha256sum -c`,
    `sh install.sh --version ${version} ${flag}`,
  ].join(" && ");
}

const LINUX_DOWNLOADS = 'cd "$(xdg-user-dir DOWNLOAD 2>/dev/null || echo ~/Downloads)"';

/** Checks a downloaded package and installs it with the distribution's package manager. */
export function linuxPackage(
  file: PinnedFile & { readonly name: string },
  kind: "deb" | "rpm",
): string {
  const manager = kind === "deb" ? "apt" : "dnf";
  return [
    LINUX_DOWNLOADS,
    `echo '${file.sha256}  ${file.name}' | sha256sum -c`,
    `sudo ${manager} install './${file.name}'`,
  ].join(" && ");
}

export type VerifySystem = "windows" | "macos" | "linux";

/** The command that checks a downloaded file against its published hash. */
export function verifyCommand(system: VerifySystem, name: string, sha256: string): string {
  if (system === "windows") {
    return `(Get-FileHash -Algorithm SHA256 -LiteralPath "$HOME\\Downloads\\${name}").Hash -eq '${sha256.toUpperCase()}'`;
  }
  if (system === "macos") return `cd ~/Downloads && echo '${sha256}  ${name}' | shasum -a 256 -c`;
  return `${LINUX_DOWNLOADS} && echo '${sha256}  ${name}' | sha256sum -c`;
}

/** Commands to open the extensions page; web pages cannot link to these addresses. */
export const EXTENSION_ADDRESSES = {
  chrome: "chrome://extensions",
  edge: "edge://extensions",
  opera: "opera://extensions",
  brave: "brave://extensions",
  firefox: "about:debugging#/runtime/this-firefox",
} as const;

export const LINUX_CARD_SERVICE = {
  deb: "sudo apt install pcscd && sudo systemctl enable --now pcscd.socket",
  rpm: "sudo dnf install pcsc-lite && sudo systemctl enable --now pcscd.socket",
} as const;

/** The commands that remove signlocal, as the install guide (docs/install.md) gives them. */
export const UNINSTALL = {
  windows: "signlocal uninstall",
  macos: "signlocal uninstall",
  linux:
    "sudo apt remove signlocal     # Debian, Ubuntu\nsudo dnf remove signlocal     # Fedora, Rocky",
} as const;
