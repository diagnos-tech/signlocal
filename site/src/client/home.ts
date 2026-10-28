// The home page's start card: marks the system this computer seems to run, writes the hint,
// and on a phone replaces the tiles with the "signlocal runs on computers" notice.
// Without JavaScript the three tiles are plain links to the wizard.

import type { MessageKey } from "../generated/message-keys.ts";
import { OS_NAMES } from "../platform/catalog.ts";
import { detect } from "../platform/detect.ts";
import { readNavigator } from "../platform/navigator.ts";
import { initDeviceNotice } from "./device-notice.ts";
import { fill, readStrings } from "./strings.ts";

export const STRINGS = ["home.start_hint_detected"] as const satisfies readonly MessageKey[];

export function initHome(): void {
  const card = document.querySelector<HTMLElement>("[data-start-card]");
  if (!card) return;
  const strings = readStrings("home", STRINGS);
  const { platform } = detect(readNavigator(navigator));
  const hint = card.querySelector<HTMLElement>("[data-start-hint]");
  const tiles = card.querySelector<HTMLElement>("[data-start-tiles]");
  const notice = card.querySelector<HTMLElement>("[data-device-notice]");

  if (platform === "mobile" && notice && tiles) {
    notice.hidden = false;
    tiles.hidden = true;
    if (hint) hint.hidden = true;
    initDeviceNotice(notice, () => notice.dataset.url ?? location.href);
    return;
  }
  if (platform === "windows" || platform === "macos" || platform === "linux") {
    card
      .querySelector<HTMLElement>(`[data-tile-os="${platform}"]`)
      ?.setAttribute("data-detected", "true");
    if (hint)
      hint.textContent = fill(strings["home.start_hint_detected"], { os: OS_NAMES[platform] });
  }
}
