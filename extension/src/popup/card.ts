/**
 * What each popup state says and offers (docs/ux.md §9 table). Pure: text
 * comes from `browser.i18n`, links from project.toml, actions are callbacks.
 */

import { t } from "./i18n";
import type { IconName } from "./icons";
import { ACTIVATE_URL, downloadUrl, type Platform, TEST_URL } from "./links";
import type { PopupState } from "./state";

/**
 * A button (runs in the popup) or a link (opens a tab). Links get the
 * "opens in a new tab" arrow; `icon` leads a button's label.
 */
export interface Action {
  readonly label: string;
  readonly icon?: IconName;
  readonly href?: string;
  readonly run?: () => void;
}

/** Everything the view needs to draw one state. */
export interface Card {
  readonly icon: IconName;
  readonly tone: "neutral" | "accent" | "success" | "warning" | "danger";
  readonly title: string;
  readonly body?: string;
  /** Technical detail in monospace (the error code), for support. */
  readonly code?: string;
  readonly primary?: Action;
  readonly secondary?: Action;
}

/** Callbacks the cards can trigger. */
export interface Handlers {
  readonly diagnostics: () => void;
  readonly retry: () => void;
}

const OS_NAME: Readonly<Record<Platform, string>> = {
  win: "Windows",
  mac: "macOS",
  linux: "Linux",
};

/**
 * The card for `state`; `os` picks the download section and its label
 * (Linux's when the platform is unknown, which only `unsupported` reaches).
 */
export function cardFor(state: PopupState, os: Platform | null, on: Handlers): Card {
  const platform = os ?? "linux";
  switch (state.kind) {
    case "checking":
      return { icon: "spinner", tone: "neutral", title: t("popup_checking") };
    case "ready":
      // Testing is what people who open a working popup want next; Diagnostics is for trouble.
      return {
        icon: "check-circle",
        tone: "success",
        title: t("popup_ready_title"),
        body: t("popup_ready_body", { version: state.appVersion }),
        primary: { label: t("popup_test"), href: TEST_URL },
        secondary: { label: t("popup_open_diagnostics"), icon: "pulse", run: on.diagnostics },
      };
    case "missing":
      return {
        icon: "download",
        tone: "accent",
        title: t("popup_missing_title"),
        body: t("popup_missing_body"),
        primary: {
          label: t("popup_download", { os: OS_NAME[platform] }),
          href: downloadUrl(platform),
        },
        secondary: { label: t("popup_activate"), href: ACTIVATE_URL },
      };
    case "outdated":
      // TODO(gustavo): "popup_update_in" with the OS's store once the app is published there.
      return {
        icon: "warning",
        tone: "warning",
        title: t("popup_outdated_title"),
        body: t("popup_outdated_body", {
          installed: state.installed || "?",
          required: state.required,
        }),
        primary: { label: t("popup_update"), href: downloadUrl(platform) },
      };
    case "error":
      return {
        icon: "x-circle",
        tone: "danger",
        title: t("popup_error_title"),
        body: t("popup_error_body"),
        code: t("popup_error_code", { code: state.code }),
        primary: { label: t("popup_retry"), icon: "retry", run: on.retry },
        secondary: { label: t("popup_redownload"), href: downloadUrl(platform) },
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
