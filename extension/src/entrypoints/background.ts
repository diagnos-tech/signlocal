/**
 * Service worker (Chromium) / event page (Firefox, Safari): the only part of
 * the extension that talks to the app. Wires the modules in ../background.
 */

import { defineBackground } from "wxt/utils/define-background";

export default defineBackground(() => {
  throw new Error("unimplemented: SPEC.md §2 (background)");
});
