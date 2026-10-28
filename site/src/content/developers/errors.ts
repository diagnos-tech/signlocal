// What each error code means to the developer who reads it. A `Record<ErrorCode, string>`, so
// adding a code to the protocol fails the type check until it is described here. The next step
// for each code comes from the SDK's own `HINTS`.

import type { ErrorCode } from "../../../../sdk/src/generated/ErrorCode.ts";

export const DESCRIPTIONS: Readonly<Record<ErrorCode, string>> = {
  ExtensionMissing:
    "The signlocal extension did not answer: it is not installed, or it is disabled for this site.",
  AppMissing:
    "The extension is there but the signlocal app is not. The person installs the app; the extension guides them.",
  AppOutdated: "The app is older than the extension needs (details has both versions).",
  ExtensionOutdated: "The extension is older than the app needs.",
  ClientOutdated: "The extension only speaks a newer protocol than your copy of @signlocal/sdk.",
  InsecureOrigin: "The page is not served over https (http://localhost also works).",
  Aborted:
    "Your AbortSignal fired, or prepare threw (your error is in cause). Nothing to show the person.",
  UserCancelled: "The person closed the window. Leave the page as it was; no message is needed.",
  Timeout: "Nobody confirmed in time, or prepare took more than 60 seconds.",
  NoCertificates: "No usable certificate was found.",
  CertificateUnavailable: "The chosen certificate is gone (the token was removed).",
  CertificateNotValid: "The certificate has expired or is not valid yet.",
  InvalidRequest:
    "A bug in the call: a wrong option, or a hash of the wrong length. The message says which.",
  UnsupportedAlgorithm: "The key cannot produce the algorithm you asked for.",
  PinIncorrect: "A wrong PIN. The app asks again, so a website rarely sees this.",
  PinLocked: "The PIN is blocked.",
  TokenRemoved: "The token was removed while signing.",
  DriverFailure: "The token driver failed; details.native has its status.",
  Busy: "Other requests are waiting in the signlocal window.",
  Internal: "A bug in signlocal.",
};
