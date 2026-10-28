// The install wizard in the browser: reads the URL and what the browser says about itself,
// runs the state machine, keeps the address bar and history in step with it, and hands the rest
// (drawing, focus, live detection) to their modules. Everything the page needs is already in
// its HTML; this only decides which parts show.

import { initDeviceNotice } from "../client/device-notice.ts";
import { fill, readStrings } from "../client/strings.ts";
import { selectTab } from "../client/tabs.ts";
import { detect } from "../platform/detect.ts";
import { readNavigator, refine } from "../platform/navigator.ts";
import { STEPS, type Step } from "../platform/types.ts";
import { readWizardData } from "./data.ts";
import { one } from "./dom.ts";
import { announce, focusField, focusStepTitle } from "./focus.ts";
import { queryFor } from "./history.ts";
import { startLiveStatus } from "./live-status.ts";
import { defaultInstallTab } from "./recipe.ts";
import { browserName, otherSystem, renderResult } from "./result-view.ts";
import { copyText, sharedUrl } from "./share.ts";
import { type Context, type Event, type Field, initial, reduce, type State } from "./state.ts";
import { STRINGS } from "./strings.ts";
import { installTestHooks } from "./test-hooks.ts";
import { parseUrl } from "./url.ts";
import { missingFields, render } from "./view.ts";

export function initWizard(): void {
  const found = document.querySelector<HTMLElement>("[data-wizard]");
  if (!found) return;
  const root: HTMLElement = found;
  const strings = readStrings("wizard", STRINGS);
  const data = readWizardData();
  const hasRelease = root.dataset.hasRelease === "yes";
  const fakeScenario = new URLSearchParams(location.search).get("sdk-fake");
  const hooks = installTestHooks(fakeScenario);

  let detection = detect(readNavigator(navigator));
  const context = (): Context => ({ detection, deviceKind: (id) => data.devices[id] });
  const parse = () => parseUrl(location.search, location.hash, (id) => data.devices[id]);

  const parsed = parse();
  let state: State = initial(parsed.choices, parsed.step, context());
  let index = 0;
  let touchedArch = parsed.choices.arch !== undefined;
  const hadParameters = location.search !== "" || location.hash !== "";

  const draw = (enteringResult = false) => {
    const urlOf = (step: Step) => `${location.pathname}${queryFor(state, step)}`;
    render({ root, strings, detection, data, urlOf }, state, hasRelease);
    renderResult(root, strings, state.choices, detection);
    const tabs = root.querySelector<HTMLElement>('[role="tablist"][data-tabs="install"]');
    if (tabs && enteringResult) selectTab(tabs, defaultInstallTab(state.choices));
  };

  let stopLive: (() => void) | undefined;
  let generation = 0;
  const syncLive = () => {
    stopLive?.();
    stopLive = undefined;
    const mine = ++generation;
    const { choices } = state;
    const here = detection.platform === choices.os;
    const sameBrowser =
      choices.browser === "chromium"
        ? detection.browser === "chromium" || detection.browser === null
        : detection.browser === choices.browser;
    if (state.step !== "result" || !here || !sameBrowser) return;
    void hooks.then(() => {
      if (mine !== generation) return;
      stopLive = startLiveStatus({
        root,
        strings,
        minAppVersion: data.minAppVersion,
        browserName: () => browserName(state.choices.browser, strings),
      });
    });
  };

  const announcement = (step: Step) =>
    fill(strings["wizard.announce"], {
      current: STEPS.indexOf(step) + 1,
      total: STEPS.length,
      name: strings[`wizard.step_${step}` as const],
    });

  function dispatch(event: Event): void {
    const before = state;
    state = reduce(state, event, context());
    if (event.type === "select" && event.field === "arch") touchedArch = true;
    const stepChanged = state.step !== before.step;
    const url = `${location.pathname}${queryFor(state)}`;
    if (stepChanged) history.pushState({ wizard: true, index: ++index }, "", url);
    else history.replaceState({ wizard: true, index }, "", url);
    draw(stepChanged && state.step === "result");
    if (stepChanged) {
      focusStepTitle(root, state.step);
      announce(root, announcement(state.step));
    } else if (state.errors[state.step] && !before.errors[state.step]) {
      const [first] = missingFields(state.step, state);
      if (first) focusField(root, first);
    }
    if (stepChanged || event.type === "select") syncLive();
  }

  history.replaceState(
    { wizard: true, index },
    "",
    hadParameters ? `${location.pathname}${queryFor(state)}` : location.href,
  );
  draw(state.step === "result");
  syncLive();

  bindControls(root, dispatch, () => {
    if (history.state?.wizard && index > 0) history.back();
    else dispatch({ type: "back" });
  });
  const share = () => sharedUrl({ ...state.choices });
  one(root, "[data-copy-wizard-link]").addEventListener("click", async () => {
    if (await copyText(share())) announce(root, strings["wizard.link_copied"]);
  });
  one(root, "[data-wrong-device-action]").addEventListener("click", () => {
    const os = otherSystem(state.choices, detection);
    if (os) dispatch({ type: "select", field: "os", value: os });
  });
  const phone = root.querySelector<HTMLElement>('[data-device-notice="phone"]');
  if (phone) initDeviceNotice(phone, share);

  window.addEventListener("popstate", () => {
    const back = parse();
    state = initial(back.choices, back.step, context());
    index = history.state?.wizard ? Number(history.state.index) : 0;
    draw(state.step === "result");
    focusStepTitle(root, state.step);
    syncLive();
  });

  // Chromium knows the real processor, and Brave announces itself, only through asynchronous
  // APIs: refine the guesses the person has not already overridden.
  void refine(navigator).then((better) => {
    if (!better.arch && !better.browser) return;
    detection = { ...detection, ...better };
    const choices = { ...state.choices };
    if (better.arch && !touchedArch && !state.provided.has("arch")) choices.arch = better.arch;
    if (better.browser && !state.provided.has("browser") && choices.os)
      choices.browser = better.browser;
    state = { ...state, choices };
    draw();
  });
}

function bindControls(root: HTMLElement, dispatch: (event: Event) => void, back: () => void): void {
  root.addEventListener("change", (event) => {
    const target = event.target as HTMLInputElement | HTMLSelectElement;
    const field = target.dataset.field as Field | undefined;
    if (field) dispatch({ type: "select", field, value: target.value });
  });
  root.addEventListener("submit", (event) => {
    event.preventDefault();
    dispatch({ type: "next" });
  });
  root.addEventListener("keydown", (event) => {
    const target = event.target as HTMLElement;
    if (event.key === "Enter" && target instanceof HTMLInputElement && target.type === "radio") {
      event.preventDefault();
      dispatch({ type: "next" });
    }
  });
  root.addEventListener("click", (event) => {
    const control = (event.target as HTMLElement).closest<HTMLElement>(
      "[data-next],[data-back],[data-restart],[data-goto],[data-step-link]",
    );
    if (!control) return;
    if (control.matches("[data-next]")) dispatch({ type: "next" });
    else if (control.matches("[data-back]")) back();
    else if (control.matches("[data-restart]")) dispatch({ type: "restart" });
    else if (control.dataset.goto) dispatch({ type: "goto", step: control.dataset.goto as Step });
    else if (control.dataset.stepLink) {
      event.preventDefault();
      dispatch({ type: "goto", step: control.dataset.stepLink as Step });
    }
  });
}
