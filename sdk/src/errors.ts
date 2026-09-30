/** Typed errors: every rejection of the SDK is a {@link WebSignError}. */

import type { ErrorCode, ErrorDetails } from "./generated/index.js";
import { HINTS } from "./hints.js";
import { HOMEPAGE } from "./project.js";

export type { ErrorCode, ErrorDetails };

/**
 * A failed SDK call. Every promise of the SDK rejects with one, never with
 * anything else.
 *
 * - `code` is stable: switch on it.
 * - `message` says what happened, `hint` what to do next; both are English,
 *   for developers, and never contain personal data.
 * - `docsUrl` explains the code on the project site.
 * - For the person, show `errorText(error, locale)` from
 *   `@websign/sdk/messages`: localized title and next step.
 *
 * @example
 * try {
 *   await sign({ hash: "SHA-256", prepare });
 * } catch (error) {
 *   if (!(error instanceof WebSignError)) throw error;
 *   if (error.code === "UserCancelled") return; // nothing to show
 *   console.warn(`${error.code}: ${error.message} ${error.hint} ${error.docsUrl}`);
 * }
 */
export class WebSignError extends Error {
  override readonly name = "WebSignError";
  /** What to do next, for the developer. People get `errorText()` instead. */
  readonly hint: string;
  /** The code's section on the project site (a stable anchor). */
  readonly docsUrl: string;

  constructor(
    /** Stable reason; one of the protocol catalog. */
    readonly code: ErrorCode,
    message: string,
    /** Versions for the `*Outdated` codes, the native status for `DriverFailure`. */
    readonly details?: ErrorDetails,
  ) {
    super(message);
    this.hint = HINTS[code];
    this.docsUrl = `${HOMEPAGE}developers.html#error-${code}`;
  }
}

/**
 * Whether `error` is a {@link WebSignError}, optionally with one of `codes`.
 * Narrows `error.code` in TypeScript, handy in a `catch (error: unknown)`.
 *
 * @example
 * catch (error) {
 *   if (isWebSignError(error, "UserCancelled", "Aborted")) return;
 *   throw error;
 * }
 */
export function isWebSignError<C extends ErrorCode = ErrorCode>(
  error: unknown,
  ...codes: readonly C[]
): error is WebSignError & { readonly code: C } {
  return error instanceof WebSignError && (!codes.length || codes.includes(error.code as C));
}

/**
 * `code` when it is in the catalog, else `Internal`: a site switching over
 * `error.code` must never meet a value outside the published union.
 */
export function knownCode(code: string): ErrorCode {
  return Object.hasOwn(HINTS, code) ? (code as ErrorCode) : "Internal";
}

/** The rejection of a call whose `AbortSignal` fired. */
export function abortedError(): WebSignError {
  return new WebSignError("Aborted", "The request was aborted through its AbortSignal.");
}

/** The rejection for a reply the SDK cannot use: a bug in the extension or the app, not the site. */
export function malformedReply(what: string): WebSignError {
  return new WebSignError("Internal", `The extension sent ${what}.`);
}
