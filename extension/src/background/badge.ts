/** The toolbar badge: "!" whenever the app is missing, outdated or not answering. */

import { browser } from "wxt/browser";

/** Shows or clears the "!" badge; never throws (the toolbar is cosmetic). */
export function showHealth(healthy: boolean): void {
  try {
    void browser.action.setBadgeText({ text: healthy ? "" : "!" });
    if (!healthy) void browser.action.setBadgeBackgroundColor({ color: "#BE242B" });
  } catch {
    // No toolbar button on this platform.
  }
}
