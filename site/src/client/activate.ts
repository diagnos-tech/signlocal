// "Finish setting up": after the person presses the button (which opens the app through its
// signlocal: link), follows the connection live and says when it worked. "Waiting" shows only
// once the button was pressed, so the page never claims to wait for something nobody started.

import type { Status } from "@signlocal/sdk";
import { onChange, status } from "@signlocal/sdk";
import type { MessageKey } from "../generated/message-keys.ts";
import { BROWSER_NAMES } from "../platform/catalog.ts";
import { detect } from "../platform/detect.ts";
import { readNavigator } from "../platform/navigator.ts";
import { installTestHooks } from "../wizard/test-hooks.ts";
import { fill, readStrings } from "./strings.ts";

export const STRINGS = [
  "activate.done",
  "browser.name_chromium",
] as const satisfies readonly MessageKey[];

export type ActivateState = "idle" | "waiting" | "done" | "extension-first";

/** What the page shows for a status, once the button has been pressed. */
export function activateState(pressed: boolean, current: Status | undefined): ActivateState {
  if (!pressed) return "idle";
  if (current?.ready) return "done";
  if (current && !current.extension.installed) return "extension-first";
  return "waiting";
}

export async function initActivate(): Promise<void> {
  const button = document.querySelector<HTMLElement>("[data-activate-button]");
  const live = document.querySelector<HTMLElement>("[data-activate-live]");
  if (!button || !live) return;
  await installTestHooks(new URLSearchParams(location.search).get("sdk-fake"));
  const strings = readStrings("activate", STRINGS);
  const { browser } = detect(readNavigator(navigator));
  const name =
    browser && browser !== "chromium" ? BROWSER_NAMES[browser] : strings["browser.name_chromium"];
  let pressed = false;
  let current: Status | undefined;

  const draw = () => {
    const state = activateState(pressed, current);
    for (const element of live.querySelectorAll<HTMLElement>("[data-state]")) {
      element.hidden = element.dataset.state !== state;
    }
    const done = live.querySelector("[data-done-text]");
    if (done) done.textContent = fill(strings["activate.done"], { browser: name });
  };

  button.addEventListener("click", () => {
    pressed = true;
    draw();
  });
  onChange((next) => {
    current = next;
    draw();
  });
  void status().then((first) => {
    current = first;
    draw();
  });
  draw();
}
