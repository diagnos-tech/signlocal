import type { ErrorCode } from "./generated/index.js";

/** The project site; the same base as `@websign/sdk`'s `docsUrl`. */
const HOMEPAGE = "https://diagnos-tech.github.io/signlocal/";

/** The code's section on the project site (a stable anchor, shared with the web SDK). */
export function docsUrlFor(code: ErrorCode): string {
  return `${HOMEPAGE}developers.html#error-${code}`;
}

/**
 * What a developer can do about each code. Typed as a `Record` so the
 * compiler fails when the protocol gains a code without advice here. Kept in
 * step with the Rust crate's `ClientError::hint`.
 */
export const HINTS: Readonly<Record<ErrorCode, string>> = {
  ExtensionMissing:
    "This code is for web pages; desktop programs do not use the browser extension.",
  AppMissing:
    "Install the SignLocal app, or pass its path with `WebSign.connect({ executable })` or WEBSIGN_EXECUTABLE. To test without the app, use `@websign/desktop/testing`.",
  AppOutdated: "The installed app is too old for this request: ask the user to update SignLocal.",
  ExtensionOutdated:
    "This code is for web pages; update the app and the browser extension together.",
  ClientOutdated: "The app speaks a newer protocol than this library: upgrade `@websign/desktop`.",
  InsecureOrigin:
    "This code is for web pages; desktop programs are never refused for their origin.",
  Aborted:
    "Your code cancelled the request (AbortSignal, or `prepare` threw). The original error is in `cause`.",
  UserCancelled:
    "The person closed the window or pressed Cancel. Not an error to report: offer to retry.",
  Timeout:
    "Nobody answered in time (the app waits 300 s for the person; 10 s for `hello`). Offer to retry.",
  NoCertificates:
    "The person has no usable certificate, or closed the window without choosing. Point them to the app's diagnostics (`openDiagnostics('certificates')`).",
  CertificateUnavailable:
    "The certificate is gone: the token was unplugged or the certificate removed. Ask the person to choose again.",
  CertificateNotValid:
    "The certificate is expired or not yet valid. Ask the person to choose another.",
  InvalidRequest:
    "The request broke a rule: a digest of the wrong length, another hash or algorithm than asked, or a malformed option. Check the `message`.",
  UnsupportedAlgorithm:
    "The chosen key cannot produce that algorithm. Pass several in `algorithm` (preferred first) or filter with `certificates({ algorithm })`.",
  PinIncorrect:
    "The PIN was wrong. The app normally retries inside its window; ask the person to try again.",
  PinLocked: "The token's PIN is blocked. The person must unblock it with the issuer's tool (PUK).",
  TokenRemoved: "The token left while signing. Ask the person to reinsert it and retry.",
  DriverFailure:
    "The OS key store or the token's driver failed. Ask the person to open Diagnostics in the app.",
  Busy: "Too many requests are waiting in the app. Wait for the current one, then retry.",
  Internal:
    "A bug in the app or the library. Please report it with the `message` (no personal data in it).",
};
