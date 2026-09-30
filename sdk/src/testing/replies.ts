/** The frames the fake sends, shaped exactly like the real content script's. */

import type {
  ErrorCode,
  ErrorDetails,
  ExtensionToPage,
  PageReply,
  Certificate as WireCertificate,
} from "../generated/index.js";

/**
 * What the fake emulates. `"ready"` is a working setup; the others are the
 * states a visitor's computer can be in.
 *
 * @example
 * installFakeWebSign({ scenario: "app-missing" });
 */
export type FakeScenario =
  | "ready"
  | "extension-missing"
  | "app-missing"
  | "app-outdated"
  | "extension-outdated"
  | "sdk-outdated";

/** The version the fake reports for itself and its app. */
export const FAKE_VERSION = "0.0.0-fake";

const EXTENSION = { version: FAKE_VERSION, browser: "chrome" } as const;
const OLD = { installed: "0.9.0", required: "1.0.0" };

export function announcement(scenario: FakeScenario): ExtensionToPage {
  const version = scenario === "sdk-outdated" ? 2 : 1;
  return {
    source: "websign-extension",
    kind: "announce",
    extension: EXTENSION,
    protocols: { min: version, max: version },
  };
}

export function errorReply(code: ErrorCode, details?: ErrorDetails): PageReply {
  return {
    type: "error",
    code,
    message: `Fake WebeSign: ${code}.`,
    ...(details && { details }),
  };
}

/** The error every request gets in a broken setup, if any. */
export function setupError(scenario: FakeScenario): PageReply | undefined {
  if (scenario === "app-missing") return errorReply("AppMissing");
  if (scenario === "app-outdated") return errorReply("AppOutdated", OLD);
  if (scenario === "extension-outdated") return errorReply("ExtensionOutdated", OLD);
  return undefined;
}

export function statusReply(scenario: FakeScenario, remembered: boolean): PageReply {
  if (scenario === "extension-outdated") return errorReply("ExtensionOutdated", OLD);
  const app = {
    version: scenario === "app-outdated" ? OLD.installed : FAKE_VERSION,
    protocols: { min: 1, max: 1 },
    os: "linux",
    arch: "x86_64",
    channel: "direct",
  } as const;
  return {
    type: "status",
    extension: EXTENSION,
    ...(scenario !== "app-missing" && { app }),
    appOutdated: scenario === "app-outdated",
    remembered,
  };
}

export const chooseReply = (certificate: WireCertificate): PageReply => ({
  type: "choose.result",
  certificates: [certificate],
});
