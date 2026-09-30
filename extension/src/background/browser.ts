/**
 * Which browser hosts the extension, for `hello`: the app records the
 * browser and its version (diagnostics, per-browser registration). Refines
 * the synchronous guess with the async APIs only the background has.
 */

import { browser } from "wxt/browser";
import {
  type Brand,
  type BrowserIdentity,
  currentBrowser,
  fromBrands,
  lowEntropyBrands,
  majorMinor,
} from "../shared/browser-name";

export type { BrowserIdentity } from "../shared/browser-name";

interface HighEntropy {
  getHighEntropyValues?(hints: string[]): Promise<{ fullVersionList?: readonly Brand[] }>;
}

async function identify(): Promise<BrowserIdentity> {
  const runtime = browser.runtime as { getBrowserInfo?: () => Promise<{ version: string }> };
  if (typeof runtime.getBrowserInfo === "function") {
    const info = await runtime.getBrowserInfo();
    return { name: "firefox", version: majorMinor(info.version) };
  }
  if (lowEntropyBrands().length > 0) {
    const data = (navigator as { userAgentData?: HighEntropy }).userAgentData;
    try {
      const detailed = await data?.getHighEntropyValues?.(["fullVersionList"]);
      if (detailed?.fullVersionList?.length) return fromBrands(detailed.fullVersionList);
    } catch {
      // Low-entropy brands still name the browser and its major version.
    }
  }
  return currentBrowser();
}

let detected: Promise<BrowserIdentity> | null = null;

/** Detected from `runtime.getBrowserInfo` (Firefox), UA-CH brands, or the UA string. */
export function detectBrowser(): Promise<BrowserIdentity> {
  detected ??= identify().catch((): BrowserIdentity => ({ name: "other", version: "0.0" }));
  return detected;
}
