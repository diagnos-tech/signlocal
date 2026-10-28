// What the wizard offers for each system: browsers, processor and distribution questions,
// certificate kinds. Display names are product names, so they are not translated.

import type { Arch, BrowserId, Os } from "./types.ts";

export const OS_NAMES: Readonly<Record<Os, string>> = {
  windows: "Windows",
  macos: "macOS",
  linux: "Linux",
};

/** "Another Chromium-based browser" reads as translated text in sentences, so it has no name here. */
export const BROWSER_NAMES: Readonly<Record<Exclude<BrowserId, "chromium">, string>> = {
  chrome: "Chrome",
  edge: "Edge",
  firefox: "Firefox",
  opera: "Opera",
  brave: "Brave",
  safari: "Safari",
};

/** The browsers shown as tiles for a system; Safari exists on the Mac only. */
export function browserTiles(os: Os): readonly Exclude<BrowserId, "chromium">[] {
  const common = ["chrome", "edge", "firefox", "opera", "brave"] as const;
  return os === "macos" ? [...common, "safari"] : common;
}

export function browserValidFor(browser: BrowserId, os: Os): boolean {
  return browser === "safari" ? os === "macos" : true;
}

/** Windows and Linux ship one build per processor; the Mac build is universal. */
export function needsArch(os: Os): boolean {
  return os !== "macos";
}

export function needsDistro(os: Os): boolean {
  return os === "linux";
}

export const DEFAULT_ARCH: Arch = "x64";
