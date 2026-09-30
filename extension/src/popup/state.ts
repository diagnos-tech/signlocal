/** The popup's states (docs/ux.md §9), decided purely from the probe result. */

import type { ProbeResult } from "../shared/runtime-messages";
import { isOlder } from "../shared/version";

/** What the popup shows. */
export type PopupState =
  | { readonly kind: "checking" }
  | { readonly kind: "ready"; readonly appVersion: string }
  | { readonly kind: "missing"; readonly os: "windows" | "macos" | "linux" }
  | { readonly kind: "outdated"; readonly installed: string; readonly required: string }
  | { readonly kind: "error"; readonly code: string }
  | { readonly kind: "unsupported" };

/** Input of {@link popupState}. */
export interface PopupFacts {
  readonly platform: string;
  /** null while the probe runs. */
  readonly probe: ProbeResult | null;
  readonly minAppVersion: string;
}

const OS_BY_PLATFORM = { win: "windows", mac: "macos", linux: "linux" } as const;

function isSupported(platform: string): platform is keyof typeof OS_BY_PLATFORM {
  return Object.hasOwn(OS_BY_PLATFORM, platform);
}

/** The state for `facts`. */
export function popupState(facts: PopupFacts): PopupState {
  const { platform, probe, minAppVersion } = facts;
  if (!isSupported(platform)) return { kind: "unsupported" };
  if (probe === null) return { kind: "checking" };
  if (probe.ok) {
    return isOlder(probe.appVersion, minAppVersion)
      ? { kind: "outdated", installed: probe.appVersion, required: minAppVersion }
      : { kind: "ready", appVersion: probe.appVersion };
  }
  // The app refused us over protocol ranges; `details` names the versions when it sent them.
  if (probe.code === "AppOutdated") {
    const installed = probe.details?.installed ?? "";
    return { kind: "outdated", installed, required: probe.details?.required ?? minAppVersion };
  }
  if (probe.code === "AppMissing") return { kind: "missing", os: OS_BY_PLATFORM[platform] };
  return { kind: "error", code: probe.code };
}
