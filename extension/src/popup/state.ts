/** The popup's states (docs/ux.md §9), decided purely from the probe result. */

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
  readonly probe:
    | null
    | { readonly ok: true; readonly appVersion: string }
    | { readonly ok: false; readonly code: string };
  readonly minAppVersion: string;
}

/** The state for `facts`. */
export function popupState(facts: PopupFacts): PopupState {
  void facts;
  throw new Error("unimplemented: SPEC.md §4");
}
