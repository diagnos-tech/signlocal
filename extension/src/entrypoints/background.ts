/**
 * Service worker (Chromium) / event page (Firefox, Safari): the only part of
 * the extension that talks to the app. Wires the modules in ../background.
 * Listeners are registered synchronously, as MV3 workers require.
 */

import { browser } from "wxt/browser";
import { defineBackground } from "wxt/utils/define-background";
import { showHealth } from "../background/badge";
import { connect } from "../background/connection";
import { handlePopupMessage } from "../background/popup-handler";
import { handleContentMessage } from "../background/relay-handler";
import { watchTabs } from "../background/tabs";
import type { HelloReason } from "../generated";
import { MIN_APP_VERSION } from "../generated";
import { isOlder } from "../shared/version";

/** Tells the app this browser has the extension, then lets the port idle out. */
function introduce(reason: HelloReason): void {
  connect(reason).then(
    (conn) => showHealth(!isOlder(conn.hello.app.version, MIN_APP_VERSION)),
    () => showHealth(false),
  );
}

export default defineBackground(() => {
  browser.runtime.onMessage.addListener((message, sender, sendResponse) => {
    if (handlePopupMessage(message, sender, sendResponse)) return true;
    handleContentMessage(message, sender);
    return false;
  });
  watchTabs();
  browser.runtime.onStartup.addListener(() => introduce("startup"));
  browser.runtime.onInstalled.addListener(() => introduce("installed"));
});
