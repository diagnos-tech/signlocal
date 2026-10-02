import type { ErrorCode, ErrorDetails } from "./generated/index.js";
import { docsUrlFor, HINTS } from "./hints.js";

/**
 * A failed call. Branch on `code`, the protocol's stable PascalCase string
 * (for example `UserCancelled` or `AppMissing`), not on `message`, which is
 * for developers and always English. `hint` says what to do about it and
 * `docsUrl` links the explanation of every code.
 *
 * @example
 * ```ts
 * try {
 *   await websign.sign({ hash: "SHA-256", prepare });
 * } catch (error) {
 *   if (!(error instanceof WebSignError)) throw error;
 *   if (error.code === "UserCancelled") return; // not a failure
 *   console.error(`${error.code}: ${error.message}\n${error.hint}\n${error.docsUrl}`);
 * }
 * ```
 */
export class WebSignError extends Error {
  override readonly name = "WebSignError";
  /** What a developer can do about this code. */
  readonly hint: string;
  /** The code's section on the project site (a stable anchor, like `@websign/sdk`). */
  readonly docsUrl: string;

  constructor(
    readonly code: ErrorCode,
    message: string,
    readonly details?: ErrorDetails,
  ) {
    super(message);
    this.hint = HINTS[code];
    this.docsUrl = docsUrlFor(code);
  }
}

/**
 * Whether `error` is a {@link WebSignError}, optionally with one of `codes`.
 * Narrows `error.code` in a `catch (error: unknown)`. Same as `@websign/sdk`.
 *
 * @example
 * ```ts
 * catch (error) {
 *   if (isWebSignError(error, "UserCancelled", "Aborted")) return;
 *   throw error;
 * }
 * ```
 */
export function isWebSignError<C extends ErrorCode = ErrorCode>(
  error: unknown,
  ...codes: readonly C[]
): error is WebSignError & { readonly code: C } {
  return error instanceof WebSignError && (codes.length === 0 || codes.includes(error.code as C));
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

/** `Aborted` for a caller's `AbortSignal` or a throwing `prepare`, the original as `cause`. */
export function aborted(cause: unknown): WebSignError {
  return withCause(new WebSignError("Aborted", "the caller cancelled the request"), cause);
}
