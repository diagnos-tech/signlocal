/** Renders the popup into `root` (plain DOM, no framework; whole popup < 15 KB). */

import { browser } from "wxt/browser";
import { MIN_APP_VERSION } from "../generated";
import type { PopupRequest, ProbeResult } from "../shared/runtime-messages";
import { type Action, type Card, cardFor, type Platform, PRIVACY_URL, t } from "./card";
import { icon } from "./icons";
import { type PopupState, popupState } from "./state";

/** Show "Checking…" only when the answer takes longer than this (docs/ux.md §9). */
const CHECKING_DELAY_MS = 150;

const PLATFORMS: readonly string[] = ["win", "mac", "linux"];

function element<K extends keyof HTMLElementTagNameMap>(
  tag: K,
  className?: string,
  text?: string,
): HTMLElementTagNameMap[K] {
  const node = document.createElement(tag);
  if (className) node.className = className;
  if (text !== undefined) node.textContent = text;
  return node;
}

function actionNode(action: Action, className: string): HTMLElement {
  if (action.href !== undefined) {
    const link = element("a", className, action.label);
    link.href = action.href;
    link.target = "_blank";
    link.rel = "noopener noreferrer";
    return link;
  }
  const button = element("button", className, action.label);
  button.type = "button";
  button.addEventListener("click", () => action.run?.());
  return button;
}

function cardNode(card: Card): HTMLElement {
  const section = element("section", "card");
  section.setAttribute("role", "status");
  const symbol = element("span", `sym ${card.tone}`);
  symbol.innerHTML = icon(card.icon, 32);
  const text = element("div", "txt");
  text.append(element("h1", undefined, card.title));
  if (card.body) text.append(element("p", "muted", card.body));
  if (card.code) text.append(element("p", "mono", card.code));
  section.append(symbol, text);
  return section;
}

function appVersionOf(state: PopupState): string {
  if (state.kind === "ready") return state.appVersion;
  return state.kind === "outdated" && state.installed ? state.installed : "—";
}

function footerNode(state: PopupState): HTMLElement {
  const footer = element("footer");
  const version = browser.runtime.getManifest().version;
  footer.append(
    element("span", undefined, t("popup_footer_versions", version, appVersionOf(state))),
  );
  const privacy = element("a", "link", t("popup_privacy"));
  privacy.href = PRIVACY_URL;
  privacy.target = "_blank";
  privacy.rel = "noopener noreferrer";
  footer.append(privacy);
  return footer;
}

function header(): HTMLElement {
  const bar = element("header");
  const mark = element("span", "brand");
  mark.innerHTML = icon("check-circle", 20);
  bar.append(mark, element("span", "name", t("extension_name")));
  return bar;
}

async function probe(): Promise<ProbeResult> {
  const request: PopupRequest = { kind: "websign-popup", op: "probe" };
  try {
    const result: ProbeResult | undefined = await browser.runtime.sendMessage(request);
    return result ?? { ok: false, code: "Internal" };
  } catch {
    return { ok: false, code: "Internal" };
  }
}

/** Probes the app through the background and renders the resulting state. */
export function render(root: HTMLElement | null): void {
  if (root === null) return;
  document.documentElement.lang = browser.i18n.getUILanguage();
  document.title = t("extension_name");

  const paint = (state: PopupState, os: Platform | null): void => {
    const card = cardFor(state, os, { diagnostics: openDiagnostics, retry: check });
    const actions = element("div", "actions");
    if (card.primary) actions.append(actionNode(card.primary, "btn"));
    if (card.secondary) actions.append(actionNode(card.secondary, "link"));
    root.replaceChildren(header(), cardNode(card), actions, footerNode(state));
    root.querySelector<HTMLElement>(".btn")?.focus({ preventScroll: true });
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
    const result = await probe();
    clearTimeout(timer);
    paint(popupState({ ...facts, probe: result }), os);
  };

  function check(): void {
    void decide();
  }

  function openDiagnostics(): void {
    const request: PopupRequest = { kind: "websign-popup", op: "diagnostics" };
    browser.runtime.sendMessage(request).then((result: ProbeResult | undefined) => {
      if (result?.ok === false) check();
      else window.close();
    }, check);
  }

  check();
}
