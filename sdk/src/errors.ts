/** Typed errors: every rejection of the SDK is a {@link WebSignError}. */

import type { ErrorCode, ErrorDetails } from "./generated/index.js";

export type { ErrorCode };

/** A failed SDK call. `code` is stable; `message` is for developers, in English. */
export class WebSignError extends Error {
  override readonly name = "WebSignError";

  constructor(
    readonly code: ErrorCode,
    message: string,
    readonly details?: ErrorDetails,
  ) {
    super(message);
  }
}

/**
 * Every code of the protocol catalog. A `Record` rather than a list so the
 * compiler fails when the generated `ErrorCode` gains or loses a member.
 */
const KNOWN: Readonly<Record<ErrorCode, 0>> = {
  ExtensionMissing: 0,
  AppMissing: 0,
  AppOutdated: 0,
  ExtensionOutdated: 0,
  ClientOutdated: 0,
  InsecureOrigin: 0,
  Aborted: 0,
  UserCancelled: 0,
  Timeout: 0,
  NoCertificates: 0,
  CertificateUnavailable: 0,
  CertificateNotValid: 0,
  InvalidRequest: 0,
  UnsupportedAlgorithm: 0,
  PinIncorrect: 0,
  PinLocked: 0,
  TokenRemoved: 0,
  DriverFailure: 0,
  Busy: 0,
  Internal: 0,
};

/**
 * `code` when it is in the catalog, else `Internal`: a site switching over
 * `error.code` must never meet a value outside the published union.
 */
export function knownCode(code: string): ErrorCode {
  return Object.hasOwn(KNOWN, code) ? (code as ErrorCode) : "Internal";
}

/** The rejection of a call whose `AbortSignal` fired. */
export function abortedError(): WebSignError {
  return new WebSignError("Aborted", "The request was aborted through its AbortSignal.");
}

/** The rejection for a reply the SDK cannot use: a bug in the extension or the app, not the site. */
export function malformedReply(what: string): WebSignError {
  return new WebSignError("Internal", `The extension sent ${what}.`);
}
