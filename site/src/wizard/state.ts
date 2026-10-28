// The wizard as a pure reducer: no DOM, no history, so every rule is testable.
//
// Detection only preselects. A value the person did not choose (or the URL did not carry) is
// never treated as confirmed: the first step shows it selected and waits for Next.

import { browserValidFor, DEFAULT_ARCH, needsArch, needsDistro } from "../platform/catalog.ts";
import {
  ARCH_IDS,
  BROWSER_IDS,
  CERT_IDS,
  type Choices,
  type Detection,
  DISTRO_IDS,
  OS_IDS,
  type Os,
  STEPS,
  type Step,
} from "../platform/types.ts";
import { type DeviceLookup, deviceFits } from "./url.ts";

export type Field = keyof Choices;

export interface State {
  readonly step: Step;
  readonly choices: Choices;
  /** Fields that came from the URL or from the person, not only from detection. */
  readonly provided: ReadonlySet<Field>;
  /** A Next press on an incomplete step. */
  readonly errors: Readonly<Partial<Record<Step, true>>>;
  /** The system change removed the chosen browser (only Safari can be: it exists on the Mac only). */
  readonly notice?: "browser-reset";
}

export type Event =
  | { readonly type: "select"; readonly field: Field; readonly value: string }
  | { readonly type: "next" }
  | { readonly type: "back" }
  | { readonly type: "goto"; readonly step: Step }
  | { readonly type: "restart" };

export interface Context {
  readonly detection: Detection;
  readonly deviceKind: DeviceLookup;
}

/** The fields each step asks for, the first one being the question itself. */
export function fieldsOf(step: Step, choices: Choices): readonly Field[] {
  const { os } = choices;
  switch (step) {
    case "system":
      return [
        "os",
        ...(os && needsArch(os) ? (["arch"] as const) : []),
        ...(os && needsDistro(os) ? (["distro"] as const) : []),
      ];
    case "browser":
      return ["browser"];
    case "certificate":
      return ["cert"];
    case "result":
      return [];
  }
}

export function complete(step: Step, choices: Choices): boolean {
  switch (step) {
    case "system":
      return fieldsOf("system", choices).every((field) => choices[field] !== undefined);
    case "browser":
      return (
        choices.os !== undefined &&
        choices.browser !== undefined &&
        browserValidFor(choices.browser, choices.os)
      );
    case "certificate":
      return choices.cert !== undefined;
    case "result":
      return true;
  }
}

const before = (step: Step) => STEPS.slice(0, STEPS.indexOf(step));

/** Whether every step before `step` is complete, so `step` may be shown. */
function reachable(step: Step, choices: Choices): boolean {
  return before(step).every((earlier) => complete(earlier, choices));
}

function without(choices: Choices, field: Field): Choices {
  const { [field]: _removed, ...rest } = choices;
  return rest;
}

/** What detection preselects for the fields the URL did not give. */
function preselect(urlChoices: Choices, detection: Detection): Choices {
  const choices: Choices = { ...urlChoices };
  const platform = detection.platform;
  const detectedOs = OS_IDS.find((os) => os === platform);
  if (!choices.os && detectedOs) choices.os = detectedOs;
  const { os } = choices;
  if (os && needsArch(os)) choices.arch ??= detection.arch ?? DEFAULT_ARCH;
  if (os === "linux" && !choices.distro && detection.distro) choices.distro = detection.distro;
  if (choices.browser && os && !browserValidFor(choices.browser, os)) delete choices.browser;
  if (!choices.browser && detection.browser && (!os || browserValidFor(detection.browser, os))) {
    choices.browser = detection.browser;
  }
  return choices;
}

export function initial(urlChoices: Choices, urlStep: Step | undefined, context: Context): State {
  const choices = preselect(urlChoices, context.detection);
  const provided = new Set<Field>(
    (Object.keys(urlChoices) as Field[]).filter((field) => choices[field] === urlChoices[field]),
  );
  const confirmed = (step: Step): boolean => {
    const [question] = fieldsOf(step, choices);
    return complete(step, choices) && (question === undefined || provided.has(question));
  };
  const first = STEPS.find((step) => step !== "result" && !confirmed(step)) ?? "result";
  const step = urlStep && reachable(urlStep, choices) ? urlStep : first;
  return { step, choices, provided, errors: {} };
}

const nextStep = (step: Step): Step =>
  STEPS[Math.min(STEPS.indexOf(step) + 1, STEPS.length - 1)] ?? step;
const previousStep = (step: Step): Step => STEPS[Math.max(STEPS.indexOf(step) - 1, 0)] ?? step;

function selectOs(state: State, os: Os, context: Context): State {
  let choices: Choices = { ...state.choices, os };
  const provided = new Set(state.provided).add("os");
  let notice: State["notice"];
  if (choices.browser && !browserValidFor(choices.browser, os)) {
    choices = without(choices, "browser");
    provided.delete("browser");
    notice = "browser-reset";
  }
  if (!needsDistro(os)) {
    choices = without(choices, "distro");
    provided.delete("distro");
  }
  if (!needsArch(os)) {
    choices = without(choices, "arch");
    provided.delete("arch");
  } else if (!choices.arch) {
    choices = { ...choices, arch: context.detection.arch ?? DEFAULT_ARCH };
  }
  const { system: _cleared, ...errors } = state.errors;
  const next = { ...state, choices, provided, errors };
  return notice ? { ...next, notice } : withoutNotice(next);
}

function withoutNotice(state: State): State {
  const { notice: _notice, ...rest } = state;
  return rest;
}

function clearError(state: State, step: Step): State {
  const { [step]: _cleared, ...errors } = state.errors;
  return withoutNotice({ ...state, errors });
}

const VALUES: Readonly<Partial<Record<Field, readonly string[]>>> = {
  arch: ARCH_IDS,
  distro: DISTRO_IDS,
  browser: BROWSER_IDS,
  cert: CERT_IDS,
};

function select(state: State, field: Field, value: string, context: Context): State {
  if (field === "browser") {
    const { os } = state.choices;
    const known = BROWSER_IDS.find((candidate) => candidate === value);
    if (!known || (os && !browserValidFor(known, os))) return state;
  } else if (field !== "os" && field !== "device" && !VALUES[field]?.includes(value)) {
    return state;
  }
  if (field === "os") {
    const os = OS_IDS.find((candidate) => candidate === value);
    return os ? selectOs(state, os, context) : state;
  }
  if (field === "device") {
    return deviceFits(context.deviceKind(value), state.choices.cert)
      ? { ...state, choices: { ...state.choices, device: value } }
      : state;
  }
  let choices: Choices = { ...state.choices, [field]: value };
  if (
    field === "cert" &&
    choices.device &&
    !deviceFits(context.deviceKind(choices.device), choices.cert)
  ) {
    choices = without(choices, "device");
  }
  const provided = new Set(state.provided).add(field);
  const step =
    STEPS.find((candidate) => fieldsOf(candidate, choices).includes(field)) ?? state.step;
  return clearError({ ...state, choices, provided }, step);
}

export function reduce(state: State, event: Event, context: Context): State {
  switch (event.type) {
    case "select":
      return select(state, event.field, event.value, context);
    case "next": {
      if (!complete(state.step, state.choices))
        return { ...state, errors: { ...state.errors, [state.step]: true } };
      const provided = new Set(state.provided);
      for (const field of fieldsOf(state.step, state.choices)) provided.add(field);
      const step = nextStep(state.step);
      // The notice that Safari was removed is for the Browser step, which is the next one.
      const moved = { ...state, step, provided, errors: {} };
      return step === "browser" ? moved : withoutNotice(moved);
    }
    case "back":
      return { ...state, step: previousStep(state.step) };
    case "goto":
      return reachable(event.step, state.choices) ? { ...state, step: event.step } : state;
    case "restart":
      return { ...initial({}, undefined, context), step: "system" };
  }
}
