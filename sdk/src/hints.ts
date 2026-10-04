/**
 * The next step for each error code, written for the developer who reads it in
 * a console or a bug report. People get localized texts from `errorText()`
 * instead: a hint may name SDK calls a person has never heard of.
 *
 * Also the catalog guard: a `Record` over `ErrorCode`, so the compiler fails
 * when the generated union gains or loses a member.
 */

import type { ErrorCode } from "./generated/index.js";

export const HINTS: Readonly<Record<ErrorCode, string>> = {
  ExtensionMissing: "Link the person to installUrl().",
  AppMissing: "The person installs the SignLocal app; the extension guides them.",
  AppOutdated: "The person updates the SignLocal app (versions in details).",
  ExtensionOutdated: "The person restarts the browser to update the extension.",
  ClientOutdated: "Update @websign/sdk.",
  InsecureOrigin: "Serve the page over https.",
  Aborted: "Your signal fired or prepare() threw (see cause).",
  UserCancelled: "The person closed the window: show nothing.",
  Timeout: "Offer a retry; keep prepare() under 60 s.",
  NoCertificates: "The person plugs in the token or card, then retries.",
  CertificateUnavailable: "The person plugs the token back in or picks another certificate.",
  CertificateNotValid: "The person picks a certificate that is valid today.",
  InvalidRequest: "Fix the call: the message names the problem.",
  UnsupportedAlgorithm: "Accept more algorithms in sign({ algorithm }).",
  PinIncorrect: "Retry: the app asks for the PIN again.",
  PinLocked: "The person unlocks the PIN with the PUK.",
  TokenRemoved: "The person plugs the token back in, then retries.",
  DriverFailure: "Retry; details.native has the driver status.",
  Busy: "Retry once the open SignLocal window closes.",
  Internal: "Report it with the app's diagnostics.",
};
