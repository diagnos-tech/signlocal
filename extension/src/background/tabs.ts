/**
 * Tab lifecycle: a closed tab or a top-level navigation to another origin
 * ends the requests made under it, so the app never keeps a window open for
 * a page that no longer exists (SPEC §2.2). `webNavigation` is deliberately
 * not requested; `tabs.onUpdated` reports the URL of tabs our content script
 * matches (its match patterns are the host permissions that allow it).
 */

import { browser } from "wxt/browser";
import { cancelNavigated, cancelTab } from "./requests";

function originOf(url: string): string | null {
  try {
    return new URL(url).origin;
  } catch {
    return null;
  }
}

/** Registers the listeners; call synchronously when the background starts. */
export function watchTabs(): void {
  browser.tabs.onRemoved.addListener((tabId) => cancelTab(tabId));
  browser.tabs.onUpdated.addListener((tabId, changeInfo) => {
    if (changeInfo.status !== "loading" || changeInfo.url === undefined) return;
    // An unparseable URL is no origin we sent requests under: cancel them all.
    cancelNavigated(tabId, originOf(changeInfo.url) ?? "null");
  });
}
