// The test page's pure parts: the sample text, which of the five setup states applies, and the
// summary a person can paste into a support request (no name, no ID number, no certificate data).

import type { Check } from "./crypto.ts";

export const SAMPLE_TEXT = "signlocal test message. This is not a document.";

export type SetupState = "checking" | "extension-missing" | "app-missing" | "outdated" | "ready";

export interface SetupStatus {
  readonly extension: { readonly installed: boolean; readonly version?: string | undefined };
  readonly app: {
    readonly installed: boolean;
    readonly version?: string | undefined;
    readonly outdated: boolean;
  };
}

/** `null` is "still looking". */
export function setupState(status: SetupStatus | null): SetupState {
  if (status === null) return "checking";
  if (!status.extension.installed) return "extension-missing";
  if (!status.app.installed) return "app-missing";
  if (status.app.outdated) return "outdated";
  return "ready";
}

export type KeyDescription =
  | { readonly type: "RSA"; readonly bits: number }
  | { readonly type: "EC"; readonly curve: string };

export function keyText(key: KeyDescription): string {
  return key.type === "RSA" ? `RSA ${key.bits}` : `EC ${key.curve}`;
}

/** What the last attempt produced: a signature (with its browser check) or an error code. */
export interface Outcome {
  readonly hash: string;
  readonly error?: string;
  readonly algorithm?: string;
  readonly key?: KeyDescription;
  readonly check?: Check;
}

export function summaryText(status: SetupStatus | null, outcome: Outcome | null): string {
  const lines = ["signlocal test", `setup: ${setupState(status)}`];
  if (status?.extension.version) lines.push(`extension: ${status.extension.version}`);
  if (status?.app.version) lines.push(`app: ${status.app.version}`);
  if (outcome) {
    lines.push(`result: ${outcome.error || outcome.check === "invalid" ? "failure" : "success"}`);
    lines.push(`error: ${outcome.error ?? "none"}`);
    lines.push(`hash: ${outcome.hash}`);
    if (outcome.algorithm && outcome.key) {
      lines.push(`algorithm: ${outcome.algorithm}`);
      lines.push(`key: ${keyText(outcome.key)}`);
      lines.push(
        `browser check: ${outcome.check === "unsupported" ? "not verified" : outcome.check}`,
      );
    }
  }
  return lines.join("\n");
}
