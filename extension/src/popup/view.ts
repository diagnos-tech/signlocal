/**
 * Renders the popup into `root` (plain DOM, no framework; whole popup < 15 KB).
 *
 * The frame (header, status region, actions, footer) is drawn at once and
 * kept: only its contents change, so the popup opens at its final width with
 * no blank frame, and the status region is a live region that already
 * exists when its text changes (screen readers skip regions born with text).
 */

import { browser } from "wxt/browser";
import { MIN_APP_VERSION } from "../generated";
import type { PopupRequest, ProbeResult } from "../shared/runtime-messages";
import { cardFor } from "./card";
import { actionNode, element, linkNode, NEW_TAB_HINT_ID, statusNodes } from "./dom";
import { t } from "./i18n";
import { icon } from "./icons";
import { type Platform, PRIVACY_URL } from "./links";
import { type PopupState, popupState } from "./state";

/** Show "Checking…" only when the answer takes longer than this (docs/ux.md §11.4). */
const CHECKING_DELAY_MS = 150;

const PLATFORMS: readonly string[] = ["win", "mac", "linux"];

async function send(op: PopupRequest["op"]): Promise<ProbeResult> {
  const request: PopupRequest = { kind: "websign-popup", op };
  try {
    const result: ProbeResult | undefined = await browser.runtime.sendMessage(request);
    return result ?? { ok: false, code: "Internal" };
  } catch {
    return { ok: false, code: "Internal" };
  }
}

/** Opens `url` in a tab and closes the popup (a no-op where the page is a tab or sidebar). */
function openTab(url: string): void {
  browser.tabs.create({ url }).then(
    () => window.close(),
    () => window.open(url, "_blank", "noopener"),
  );
}

function header(): HTMLElement {
  const bar = element("header");
  const mark = element("span", "brand");
  mark.innerHTML = icon("brand", 20);
  bar.append(mark, element("span", undefined, t("extension_name")));
  return bar;
}

function versionsText(app: string | null): string {
  const ext = browser.runtime.getManifest().version;
  return app ? t("popup_footer_versions", { app, ext }) : t("popup_footer_extension", { ext });
}

function appVersionOf(state: PopupState): string | null {
  if (state.kind === "ready") return state.appVersion;
  return state.kind === "outdated" && state.installed ? state.installed : null;
}

/** Probes the app through the background and renders the resulting state. */
export function render(root: HTMLElement | null): void {
  if (root === null) return;
  document.documentElement.lang = browser.i18n.getUILanguage();
  document.title = t("extension_name");

  const status = element("section", "status");
  status.setAttribute("role", "status");
  const actions = element("div", "actions");
  const versions = element("span", undefined, versionsText(null));
  const footer = element("footer");
  footer.append(versions, linkNode(t("popup_privacy"), PRIVACY_URL, "link", openTab, false));
  const hint = element("span", undefined, t("popup_new_tab"));
  hint.id = NEW_TAB_HINT_ID;
  hint.hidden = true;
  root.replaceChildren(header(), status, actions, footer, hint);
  // Focus that sat on an action the next paint removes (Try again) moves to the new primary.
  let refocus = false;

  const paint = (state: PopupState, os: Platform | null): void => {
    const card = cardFor(state, os, { diagnostics: openDiagnostics, retry: check });
    refocus ||= actions.contains(document.activeElement);
    status.replaceChildren(...statusNodes(card));
    status.setAttribute("aria-busy", String(state.kind === "checking"));
    actions.removeAttribute("aria-busy");
    actions.replaceChildren();
    if (card.primary) actions.append(actionNode(card.primary, "btn", openTab));
    if (card.secondary) actions.append(actionNode(card.secondary, "btn ghost", openTab));
    versions.textContent = versionsText(appVersionOf(state));
    const first = actions.querySelector<HTMLElement>(".btn");
    if (refocus && first) {
      first.focus();
      refocus = false;
    }
  };

  const decide = async (): Promise<void> => {
    const { os: platform } = await browser.runtime.getPlatformInfo();
    const os = PLATFORMS.includes(platform) ? (platform as Platform) : null;
    const facts = { platform, minAppVersion: MIN_APP_VERSION };
    if (os === null) return paint(popupState({ ...facts, probe: null }), os);
    const timer = setTimeout(
      () => paint(popupState({ ...facts, probe: null }), os),
      CHECKING_DELAY_MS,
    );
    const result = await send("probe");
    clearTimeout(timer);
    paint(popupState({ ...facts, probe: result }), os);
  };

  function check(): void {
    void decide();
  }

  /** Diagnostics is the app's window: close on success, re-check if the app could not open it. */
  function openDiagnostics(): void {
    actions.setAttribute("aria-busy", "true");
    void send("diagnostics").then((result) => {
      if (result.ok) window.close();
      else check();
    });
  }

  check();
}
