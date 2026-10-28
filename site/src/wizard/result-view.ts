// The parts of "Your steps" that name the choices: the title, and the notice shown when the
// steps are for another computer than this one.

import { fill } from "../client/strings.ts";
import { BROWSER_NAMES, OS_NAMES } from "../platform/catalog.ts";
import type { BrowserId, Choices, Detection, Os } from "../platform/types.ts";
import { one } from "./dom.ts";
import type { WizardStrings } from "./view.ts";

export function browserName(browser: BrowserId | undefined, strings: WizardStrings): string {
  return browser === undefined || browser === "chromium"
    ? strings["browser.name_chromium"]
    : BROWSER_NAMES[browser];
}

/** The detected system when it differs from the chosen one (a shared link, another computer). */
export function otherSystem(choices: Choices, detection: Detection): Os | undefined {
  const { platform } = detection;
  const known = platform === "windows" || platform === "macos" || platform === "linux";
  return known && choices.os !== undefined && platform !== choices.os ? platform : undefined;
}

export function renderResult(
  root: HTMLElement,
  strings: WizardStrings,
  choices: Choices,
  detection: Detection,
): void {
  one(root, "[data-result-title]").textContent = fill(strings["result.title"], {
    os: choices.os ? OS_NAMES[choices.os] : "",
    browser: browserName(choices.browser, strings),
  });
  const banner = one(root, "[data-wrong-device]");
  const other = otherSystem(choices, detection);
  banner.hidden = other === undefined;
  if (!other) return;
  one(banner, "[data-wrong-device-text]").textContent = fill(strings["result.wrong_device"], {
    os: choices.os ? OS_NAMES[choices.os] : "",
    detected: OS_NAMES[other],
  });
  const action = one(banner, "[data-wrong-device-action]");
  action.textContent = fill(strings["result.wrong_device_action"], { detected: OS_NAMES[other] });
}
