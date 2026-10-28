// Draws a wizard state onto the page that Astro rendered: which step shows, what is selected,
// which pills, hints, notices and conditional blocks apply, the progress indicator and the
// navigation bar. It only changes attributes and text of elements that already exist.

import type { Strings } from "../client/strings.ts";
import { fill } from "../client/strings.ts";
import { browserValidFor } from "../platform/catalog.ts";
import type { Detection, Step } from "../platform/types.ts";
import { STEPS } from "../platform/types.ts";
import { apply, type Facts } from "./conditions.ts";
import type { WizardData } from "./data.ts";
import { all, one } from "./dom.ts";
import { complete, type Field, fieldsOf, type State } from "./state.ts";
import type { STRINGS } from "./strings.ts";

export type WizardStrings = Strings<(typeof STRINGS)[number]>;

export interface ViewContext {
  readonly root: HTMLElement;
  readonly strings: WizardStrings;
  readonly detection: Detection;
  readonly data: WizardData;
  /** The URL of a step with the confirmed choices, for the stepper's links. */
  readonly urlOf: (step: Step) => string;
}

const STEP_NAMES = {
  system: "wizard.step_system",
  browser: "wizard.step_browser",
  certificate: "wizard.step_certificate",
  result: "wizard.step_result",
} as const;

export function factsOf(state: State, data: WizardData, hasRelease: boolean): Facts {
  const { choices } = state;
  return {
    ...(choices.os && { os: choices.os }),
    ...(choices.arch && { arch: choices.arch }),
    ...(choices.distro && { distro: choices.distro }),
    ...(choices.browser && { browser: choices.browser, channel: data.channels[choices.browser] }),
    ...(choices.cert && { cert: choices.cert }),
    release: hasRelease ? "yes" : "no",
  };
}

/** Which fields of a step the person left empty after pressing Next. */
export function missingFields(step: Step, state: State): Field[] {
  const { choices } = state;
  if (step === "browser") {
    return choices.browser && choices.os && browserValidFor(choices.browser, choices.os)
      ? []
      : ["browser"];
  }
  return fieldsOf(step, choices).filter((field) => choices[field] === undefined);
}

function syncInputs(root: HTMLElement, state: State): void {
  for (const input of all<HTMLInputElement>(root, 'input[type="radio"][data-field]')) {
    const field = input.dataset.field as Field;
    input.checked = state.choices[field] === input.value;
  }
  for (const select of all<HTMLSelectElement>(root, "select[data-field]")) {
    select.value = state.choices.device ?? "";
  }
}

function syncPills(root: HTMLElement, detection: Detection): void {
  const detected = new Set<string>();
  if (
    detection.platform === "windows" ||
    detection.platform === "macos" ||
    detection.platform === "linux"
  ) {
    detected.add(`os:${detection.platform}`);
  }
  if (detection.arch) detected.add(`arch:${detection.arch}`);
  for (const pill of all(root, "[data-detected]"))
    pill.hidden = !detected.has(pill.dataset.detected ?? "");
}

/** Shows the message of every question left empty and ties it to the options it is about. */
function syncErrors(root: HTMLElement, state: State): void {
  for (const step of ["system", "browser", "certificate"] as const) {
    const missing = state.errors[step] === true ? missingFields(step, state) : [];
    const area = one(root, `[data-step="${step}"]`);
    const messages = all(area, "[data-error]");
    for (const message of messages)
      message.hidden = !missing.includes(message.dataset.error as Field);
    for (const input of all<HTMLInputElement>(area, "input[data-field]")) {
      const message = messages.find(
        (candidate) => !candidate.hidden && candidate.dataset.error === input.dataset.field,
      );
      if (message) {
        message.id ||= `error-${message.dataset.error}`;
        input.setAttribute("aria-invalid", "true");
        input.setAttribute("aria-describedby", message.id);
      } else {
        input.removeAttribute("aria-invalid");
        input.removeAttribute("aria-describedby");
      }
    }
  }
}

function syncProgress(context: ViewContext, state: State): void {
  const { root, strings } = context;
  const current = STEPS.indexOf(state.step);
  for (const item of all(root, "[data-step-item]")) {
    const step = item.dataset.stepItem as Step;
    const index = STEPS.indexOf(step);
    const done = index < current && complete(step, state.choices);
    item.dataset.state = index === current ? "current" : done ? "done" : "future";
    const link = one<HTMLAnchorElement>(item, "[data-step-link]");
    link.href = context.urlOf(step);
    if (index === current) link.setAttribute("aria-current", "step");
    else link.removeAttribute("aria-current");
    link.tabIndex = done ? 0 : -1;
  }
  for (const bar of all(root, "[data-bar]")) {
    bar.dataset.on = String(STEPS.indexOf(bar.dataset.bar as Step) <= current);
  }
  one(root, "[data-counter]").textContent =
    `${fill(strings["wizard.counter"], { current: current + 1, total: STEPS.length })} · ${strings[STEP_NAMES[state.step]]}`;
}

function syncNavigation(root: HTMLElement, state: State): void {
  const nav = one(root, "[data-nav]");
  nav.hidden = state.step === "result";
  one(nav, "[data-back]").hidden = state.step === "system";
  one(nav, '[data-next-text="next"]').hidden = state.step === "certificate";
  one(nav, '[data-next-text="show"]').hidden = state.step !== "certificate";
}

function syncDeviceInfo(root: HTMLElement, state: State): void {
  for (const device of all(root, "[data-device-id]")) {
    device.hidden = device.dataset.deviceId !== state.choices.device;
  }
}

function syncDeviceNotices(root: HTMLElement, detection: Detection): void {
  const kind = detection.platform === "mobile" ? "phone" : detection.platform;
  for (const notice of all(root, "[data-device-notice]"))
    notice.hidden = notice.dataset.deviceNotice !== kind;
  const known =
    detection.platform === "windows" ||
    detection.platform === "macos" ||
    detection.platform === "linux";
  one(root, '[data-hint="detected"]').hidden = !known;
  one(root, '[data-hint="unknown"]').hidden = known;
}

/** Draws the whole state. Cheap enough to call on every change. */
export function render(context: ViewContext, state: State, hasRelease: boolean): void {
  const { root, data, detection } = context;
  root.dataset.ready = "";
  for (const step of all(root, ".wizard__step")) step.hidden = step.dataset.step !== state.step;
  syncInputs(root, state);
  syncPills(root, detection);
  syncDeviceNotices(root, detection);
  apply(root, factsOf(state, data, hasRelease));
  for (const notice of all(root, "[data-notice]"))
    notice.hidden = notice.hidden || state.notice === undefined;
  syncDeviceInfo(root, state);
  syncErrors(root, state);
  syncProgress(context, state);
  syncNavigation(root, state);
}
