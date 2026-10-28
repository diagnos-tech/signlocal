// Live detection on "Your steps": asks the SDK whether the extension and the app are already
// there (it never opens a window) and shows chips on the cards, then the "ready" banner.
// It runs only when this computer is the one the steps are for.

import type { Status } from "@signlocal/sdk";
import { onChange, status } from "@signlocal/sdk";
import { fill, type Strings } from "../client/strings.ts";
import { all, one } from "./dom.ts";
import type { STRINGS } from "./strings.ts";

type WizardStrings = Strings<(typeof STRINGS)[number]>;

export interface LiveContext {
  readonly root: HTMLElement;
  readonly strings: WizardStrings;
  /** project.toml's minimum app version, shown when the installed app is older. */
  readonly minAppVersion: string;
  readonly browserName: () => string;
}

const CHIPS: Readonly<Record<string, string>> = {
  "extension-installed": "live.extension_installed",
  "extension-missing": "live.not_yet",
  "app-installed": "live.app_installed",
  "app-outdated": "live.app_outdated",
  connected: "live.connected",
};

function show(root: HTMLElement, name: string, visible: boolean, text = ""): void {
  for (const chip of all(root, `[data-live="${name}"]`)) {
    chip.hidden = !visible;
    const label = chip.querySelector("[data-live-text]");
    if (label && visible) label.textContent = text;
  }
}

/** The chips and the banner for one status; pure with respect to the page's strings. */
export function renderStatus(context: LiveContext, current: Status | undefined): void {
  const { root, strings } = context;
  const ready = current?.ready === true;
  const extension = current?.extension.installed === true;
  const app = current?.app.installed === true;
  const outdated = current?.app.outdated === true;
  const text = (key: keyof typeof CHIPS, variables: Record<string, string> = {}) =>
    fill(strings[CHIPS[key] as keyof WizardStrings], variables);
  show(
    root,
    "extension-installed",
    extension,
    text("extension-installed", { version: current?.extension.version ?? "" }),
  );
  show(root, "extension-missing", current !== undefined && !extension, text("extension-missing"));
  show(
    root,
    "app-installed",
    extension && app && !outdated,
    text("app-installed", { version: current?.app.version ?? "" }),
  );
  show(
    root,
    "app-outdated",
    extension && outdated,
    text("app-outdated", {
      installed: current?.app.version ?? "?",
      required: context.minAppVersion,
    }),
  );
  show(root, "connected", ready, text("connected"));
  const banner = one(root, "[data-ready-banner]");
  banner.hidden = !ready;
  one(banner, "[data-ready-title]").textContent = fill(strings["result.ready_title"], {
    browser: context.browserName(),
  });
  collapseWhenReady(root, ready);
}

/** Once everything works, the install, extension and connect cards fold their steps away. */
function collapseWhenReady(root: HTMLElement, ready: boolean): void {
  if (root.hasAttribute("data-ready-state") === ready) return;
  root.toggleAttribute("data-ready-state", ready);
  for (const folded of all<HTMLDetailsElement>(root, "[data-again]")) {
    folded.toggleAttribute("data-collapsible", ready);
    folded.open = !ready;
  }
}

/** Starts listening; returns the function that stops. */
export function startLiveStatus(context: LiveContext): () => void {
  let stopped = false;
  void status().then((first) => {
    if (!stopped) renderStatus(context, first);
  });
  const unsubscribe = onChange((next) => {
    if (!stopped) renderStatus(context, next);
  });
  return () => {
    stopped = true;
    unsubscribe();
    renderStatus(context, undefined);
  };
}
