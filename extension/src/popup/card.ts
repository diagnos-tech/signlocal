/**
 * What each popup state says and offers (docs/ux.md §9 table). Pure: text
 * comes from `browser.i18n`, links from project.toml, actions are callbacks.
 */

import { browser } from "wxt/browser";
import { HOMEPAGE } from "../generated";
import type { IconName } from "./icons";
import type { PopupState } from "./state";

/** A button or link under the status card. */
export interface Action {
  readonly label: string;
  readonly href?: string;
  readonly run?: () => void;
}

/** Everything the view needs to draw one state. */
export interface Card {
  readonly icon: IconName;
  readonly tone: "neutral" | "accent" | "success" | "warning" | "danger";
  readonly title: string;
  readonly body?: string;
  /** Technical detail in monospace (the error code). */
  readonly code?: string;
  readonly primary?: Action;
  readonly secondary?: Action;
}

/** Desktop OS the popup can name, from `runtime.getPlatformInfo().os`. */
export type Platform = "win" | "mac" | "linux";

/** Localized text; falls back to the key so a missing string is visible, not blank. */
export function t(key: string, ...substitutions: string[]): string {
  return browser.i18n.getMessage(key as never, substitutions) || key;
}

const OS_NAME: Readonly<Record<Platform, string>> = {
  win: "Windows",
  mac: "macOS",
  linux: "Linux",
};
const STORE_KEY: Readonly<Record<Platform, string>> = {
  win: "store_microsoft",
  mac: "store_apple",
  linux: "store_linux",
};

// TODO(gustavo): per-store URLs once the app is published; until then every path is the site.
/** The site's download page (every OS; the stores come later). */
export const DOWNLOAD_URL = `${HOMEPAGE}download.html`;
/**
 * Where people finish activating an installed app (docs/ux.md §9): the
 * site's test page follows the extension's announcement live and can open
 * the app through its URL scheme, which stays open while the popup cannot.
 */
export const ACTIVATE_URL = `${HOMEPAGE}activate/`;
/** The privacy notice. */
export const PRIVACY_URL = `${HOMEPAGE}privacy.html`;

/** Callbacks the cards can trigger. */
export interface Handlers {
  readonly diagnostics: () => void;
  readonly retry: () => void;
}

/**
 * The card for `state`; `os` picks the download and store wording (the generic
 * download page when the platform is unknown).
 */
export function cardFor(state: PopupState, os: Platform | null, on: Handlers): Card {
  const platform = os ?? "linux";
  switch (state.kind) {
    case "checking":
      return { icon: "spinner", tone: "neutral", title: t("popup_checking") };
    case "ready":
      return {
        icon: "check-circle",
        tone: "success",
        title: t("popup_ready_title"),
        body: t("popup_ready_body", state.appVersion),
        primary: { label: t("popup_open_diagnostics"), run: on.diagnostics },
      };
    case "missing":
      return {
        icon: "download-simple",
        tone: "accent",
        title: t("popup_missing_title"),
        body: t("popup_missing_body"),
        primary: { label: t("popup_download", OS_NAME[platform]), href: DOWNLOAD_URL },
        secondary: { label: t("popup_activate"), href: ACTIVATE_URL },
      };
    case "outdated":
      return {
        icon: "warning",
        tone: "warning",
        title: t("popup_outdated_title"),
        body: t("popup_outdated_body", state.installed || "?", state.required),
        primary: {
          label: t("popup_update_in", t(STORE_KEY[platform])),
          href: DOWNLOAD_URL,
        },
      };
    case "error":
      return {
        icon: "x-circle",
        tone: "danger",
        title: t("popup_error_title"),
        body: t("popup_error_body"),
        code: state.code,
        primary: { label: t("popup_retry"), run: on.retry },
        secondary: { label: t("popup_redownload"), href: DOWNLOAD_URL },
      };
    case "unsupported":
      return {
        icon: "info",
        tone: "neutral",
        title: t("popup_unsupported_title"),
        body: t("popup_unsupported_body"),
      };
  }
}
