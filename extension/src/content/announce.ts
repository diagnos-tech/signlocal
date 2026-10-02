/**
 * Announces the extension to the page at start, on load and on `discover`.
 * Synchronous on purpose: the SDK gives up after 1 s without an announcement,
 * so the browser name comes from what the page's context can read at once.
 */

import { browser } from "wxt/browser";
import type { ExtensionToPage } from "../generated";
import { currentBrowser } from "../shared/browser-name";
import { EXTENSION_SOURCE, PROTOCOL_VERSION } from "../shared/limits";

/** Posts one announcement to the page's own origin. */
export function announce(): void {
  const message: ExtensionToPage = {
    kind: "announce",
    source: EXTENSION_SOURCE,
    extension: { version: browser.runtime.getManifest().version, browser: currentBrowser().name },
    protocols: { min: PROTOCOL_VERSION, max: PROTOCOL_VERSION },
  };
  window.postMessage(message, location.origin);
}
