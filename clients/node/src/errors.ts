import type { ErrorCode, ErrorDetails } from "./generated/index.js";

/**
 * A failed call. `code` is the protocol's stable PascalCase string (for
 * example `UserCancelled` or `AppMissing`), so callers branch on it instead of
 * parsing `message`, which is for developers and always English.
 */
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

/** Every code the app may send; the compiler keeps this in step with the generated type. */
const KNOWN_CODES: Record<ErrorCode, true> = {
  ExtensionMissing: true,
  AppMissing: true,
  AppOutdated: true,
  ExtensionOutdated: true,
  ClientOutdated: true,
  InsecureOrigin: true,
  Aborted: true,
  UserCancelled: true,
  Timeout: true,
  NoCertificates: true,
  CertificateUnavailable: true,
  CertificateNotValid: true,
  InvalidRequest: true,
  UnsupportedAlgorithm: true,
  PinIncorrect: true,
  PinLocked: true,
  TokenRemoved: true,
  DriverFailure: true,
  Busy: true,
  Internal: true,
};

/**
 * Turns an `error` frame into a {@link WebSignError}. A code this library does
 * not know (an app newer than the catalog) becomes `Internal` rather than
 * leaking a value callers cannot switch over.
 */
export function fromWire(wire: {
  code?: unknown;
  message?: unknown;
  details?: unknown;
}): WebSignError {
  const message = typeof wire.message === "string" ? wire.message : "the app reported an error";
  const details =
    typeof wire.details === "object" && wire.details !== null
      ? (wire.details as ErrorDetails)
      : undefined;
  if (typeof wire.code === "string" && Object.hasOwn(KNOWN_CODES, wire.code)) {
    return new WebSignError(wire.code as ErrorCode, message, details);
  }
  return new WebSignError("Internal", message, details);
}

/** Attaches the original failure so callers can inspect what a `prepare` callback threw. */
export function withCause(error: WebSignError, cause: unknown): WebSignError {
  error.cause = cause;
  return error;
}
