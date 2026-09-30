/**
 * Runs in every https page (and http://localhost), in all frames, at
 * document_start: announces the extension and relays page messages to the
 * background. Wires the modules in ../content.
 */

import { defineContentScript } from "wxt/utils/define-content-script";
import { announce } from "../content/announce";
import { startRelay } from "../content/relay";

export default defineContentScript({
  matches: ["https://*/*", "http://localhost/*", "http://127.0.0.1/*", "http://[::1]/*"],
  runAt: "document_start",
  allFrames: true,
  main() {
    startRelay();
    announce();
    window.addEventListener("load", announce);
  },
});
