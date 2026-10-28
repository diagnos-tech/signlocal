// Guesses the visitor's system, processor and browser from what the browser says about itself.
// Pure: the page builds the input from `navigator`, the tests from fixtures. Nothing is sent
// anywhere, and every guess is only a preselection the person confirms.

import type { Arch, BrowserId, Detection, Distro, Platform } from "./types.ts";

export interface DetectionInput {
  readonly userAgent: string;
  readonly maxTouchPoints: number;
  /** navigator.userAgentData.platform, Chromium only. */
  readonly uaDataPlatform?: string;
  readonly uaDataMobile?: boolean;
  readonly uaBrands?: readonly string[];
  /** navigator.brave.isBrave(), which answers asynchronously. */
  readonly isBrave?: boolean;
}

const MAC = /Macintosh|Mac OS X/;

export function detectPlatform(input: DetectionInput): Platform {
  const { userAgent: ua, uaDataPlatform: hint } = input;
  if (input.uaDataMobile === true) return "mobile";
  if (/Android|iPhone|iPad|iPod/i.test(ua)) return "mobile";
  if (MAC.test(ua) && input.maxTouchPoints > 1) return "mobile";
  if (/CrOS/.test(ua) || hint === "Chrome OS") return "chromeos";
  if (/Windows/.test(ua) || hint === "Windows") return "windows";
  if (MAC.test(ua) || hint === "macOS") return "macos";
  if (/Linux|X11/.test(ua) || hint === "Linux") return "linux";
  return null;
}

export function detectBrowser(input: DetectionInput): BrowserId | null {
  const { userAgent: ua } = input;
  const brands = input.uaBrands ?? [];
  const brand = (name: string) => brands.some((entry) => entry.includes(name));
  if (input.isBrave === true) return "brave";
  if (/Edg\//.test(ua) || brand("Microsoft Edge")) return "edge";
  if (/OPR\/|Opera/.test(ua) || brand("Opera")) return "opera";
  if (/Firefox\/|FxiOS/.test(ua)) return "firefox";
  if (/Vivaldi|YaBrowser/.test(ua)) return "chromium";
  if (brands.length > 0 && !brand("Google Chrome") && brand("Chromium")) return "chromium";
  if (/Chrome\/|Chromium\//.test(ua)) return "chrome";
  if (/Version\/.*Safari\//.test(ua)) return "safari";
  return null;
}

export function detectArch(input: DetectionInput, platform: Platform): Arch | null {
  if (platform === "macos") return null;
  const { userAgent: ua } = input;
  if (/aarch64|arm64|ARM64/.test(ua)) return "arm64";
  if (/x86_64|x64|Win64|WOW64|amd64/.test(ua)) return "x64";
  return null;
}

/** A weak hint: the user agent of most Linux browsers names the distribution only sometimes. */
export function detectDistro(input: DetectionInput): Distro | null {
  const { userAgent: ua } = input;
  if (/Ubuntu|Debian|Mint/.test(ua)) return "deb";
  if (/Fedora|Red Hat|CentOS|Rocky/.test(ua)) return "rpm";
  return null;
}

export function detect(input: DetectionInput): Detection {
  const platform = detectPlatform(input);
  return {
    platform,
    arch: detectArch(input, platform),
    browser: detectBrowser(input),
    distro: platform === "linux" ? detectDistro(input) : null,
  };
}

/** The high-entropy client hints the refinement asks for (Chromium only). */
export interface HighEntropyValues {
  readonly architecture?: string;
  readonly bitness?: string;
}

/** "arm" → arm64, "x86" with 64 bits → x64; anything else says nothing. */
export function archFromHints(values: HighEntropyValues): Arch | null {
  if (values.architecture === "arm") return "arm64";
  if (values.architecture === "x86" && values.bitness === "64") return "x64";
  return null;
}
