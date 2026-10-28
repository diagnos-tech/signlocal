// The texts the wizard needs in the browser, beyond what the page already contains: the titles
// and notices that name the chosen system and browser, and the live status chips.

import type { MessageKey } from "../generated/message-keys.ts";

export const STRINGS = [
  "wizard.counter",
  "wizard.announce",
  "wizard.step_system",
  "wizard.step_browser",
  "wizard.step_certificate",
  "wizard.step_result",
  "wizard.link_copied",
  "browser.name_chromium",
  "result.title",
  "result.wrong_device",
  "result.wrong_device_action",
  "result.ready_title",
  "result.show_again",
  "live.checking",
  "live.extension_installed",
  "live.app_installed",
  "live.app_outdated",
  "live.connected",
  "live.not_yet",
] as const satisfies readonly MessageKey[];
