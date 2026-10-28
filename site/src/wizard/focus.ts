// Where focus goes: to the new step's heading when a step opens, to the first radio of a group
// the person left empty, and what a screen reader is told.

import type { Step } from "../platform/types.ts";
import { all, one } from "./dom.ts";
import type { Field } from "./state.ts";

const stepElement = (root: HTMLElement, step: Step) => one(root, `[data-step="${step}"]`);

export function focusStepTitle(root: HTMLElement, step: Step): void {
  one(stepElement(root, step), "[data-step-title]").focus({ preventScroll: false });
}

/** Focuses the first radio of a field's group, so a screen reader reads the group's error. */
export function focusField(root: HTMLElement, field: Field): void {
  const first = all<HTMLInputElement>(root, `input[data-field="${field}"]`).find(
    (input) => input.offsetParent !== null,
  );
  first?.focus();
}

/** The polite live region under the card. */
export function announce(root: HTMLElement, message: string): void {
  one(root, "[data-announce]").textContent = message;
}
