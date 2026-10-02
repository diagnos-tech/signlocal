/**
 * Names the browser from what every context can read synchronously: UA-CH
 * low-entropy brands (Chromium family) or the user agent string (Firefox,
 * Safari). The content script announces with this at `document_start`, when
 * waiting for an async API would delay the page's discovery.
 */

import type { BrowserName } from "../generated";

/** A browser and its `major.minor` version. */
export interface BrowserIdentity {
  readonly name: BrowserName;
  readonly version: string;
}

/** One UA-CH brand entry. */
export interface Brand {
  readonly brand: string;
  readonly version: string;
}

/** Brand → name, most specific first: every Chromium browser also says "Chromium". */
const BRANDS: readonly (readonly [string, BrowserName])[] = [
  ["Microsoft Edge", "edge"],
  ["Brave", "brave"],
  ["Opera", "opera"],
  ["Vivaldi", "vivaldi"],
  ["Google Chrome", "chrome"],
];

/** "129.0.6668.58" → "129.0"; a bare "129" → "129.0". */
export function majorMinor(version: string): string {
  const [major = "0", minor = "0"] = version.split(".");
  return `${major}.${minor}`;
}

/** The Chromium-family browser named by `brands`; plain "chromium" when none is known. */
export function fromBrands(brands: readonly Brand[]): BrowserIdentity {
  for (const [brand, name] of BRANDS) {
    const found = brands.find((entry) => entry.brand === brand);
    if (found) return { name, version: majorMinor(found.version) };
  }
  const base = brands.find((entry) => entry.brand === "Chromium");
  return { name: "chromium", version: majorMinor(base?.version ?? "0") };
}

function versionIn(pattern: RegExp, text: string): string {
  return majorMinor(pattern.exec(text)?.[1] ?? "0");
}

/** Firefox or Safari from the user agent string; anything else is "other". */
export function fromUserAgent(ua: string): BrowserIdentity {
  if (/Firefox\//.test(ua)) return { name: "firefox", version: versionIn(/Firefox\/([\d.]+)/, ua) };
  if (/Safari\//.test(ua) && !/Chrom(e|ium)|Edg/.test(ua)) {
    return { name: "safari", version: versionIn(/Version\/([\d.]+)/, ua) };
  }
  return { name: "other", version: "0.0" };
}

/** The navigator's low-entropy brands, when the browser exposes UA-CH. */
export function lowEntropyBrands(): readonly Brand[] {
  const nav = globalThis.navigator as { userAgentData?: { brands?: readonly Brand[] } } | undefined;
  return nav?.userAgentData?.brands ?? [];
}

/** The browser, synchronously; the version is the major only on Chromium. */
export function currentBrowser(): BrowserIdentity {
  const brands = lowEntropyBrands();
  if (brands.length > 0) return fromBrands(brands);
  return fromUserAgent(globalThis.navigator?.userAgent ?? "");
}
