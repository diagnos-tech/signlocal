// The vocabulary of the install wizard, shared by the build and the browser.

export type Os = "windows" | "macos" | "linux";
/** Windows and Linux only: macOS builds are universal. */
export type Arch = "x64" | "arm64";
/** Linux only. */
export type Distro = "deb" | "rpm" | "other";
export type BrowserId = "chrome" | "edge" | "firefox" | "opera" | "brave" | "safari" | "chromium";
export type CertKind = "token" | "card" | "file" | "unsure";
export type Step = "system" | "browser" | "certificate" | "result";

export const OS_IDS: readonly Os[] = ["windows", "macos", "linux"];
export const ARCH_IDS: readonly Arch[] = ["x64", "arm64"];
export const DISTRO_IDS: readonly Distro[] = ["deb", "rpm", "other"];
export const BROWSER_IDS: readonly BrowserId[] = [
  "chrome",
  "edge",
  "firefox",
  "opera",
  "brave",
  "safari",
  "chromium",
];
export const CERT_IDS: readonly CertKind[] = ["token", "card", "file", "unsure"];
export const STEPS: readonly Step[] = ["system", "browser", "certificate", "result"];

export interface Choices {
  os?: Os;
  arch?: Arch;
  distro?: Distro;
  browser?: BrowserId;
  cert?: CertKind;
  /** A devices.json id whose kind matches `cert`. */
  device?: string;
}

/** What detection may say about the computer. */
export type Platform = Os | "mobile" | "chromeos" | null;

export interface Detection {
  readonly platform: Platform;
  readonly arch: Arch | null;
  readonly browser: BrowserId | null;
  readonly distro: Distro | null;
}

/** Whether the extension comes from a store or is added by hand. */
export type Channel = "store" | "dev";
