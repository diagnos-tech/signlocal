// How a browser is named in a sentence: its product name, or "your browser" for the
// "another Chromium-based browser" choice.

import type { MessageKey } from "../generated/message-keys.ts";
import { BROWSER_NAMES } from "../platform/catalog.ts";
import type { BrowserId } from "../platform/types.ts";

export function browserLabel(id: BrowserId, t: (key: MessageKey) => string): string {
  return id === "chromium" ? t("browser.name_chromium") : BROWSER_NAMES[id];
}
