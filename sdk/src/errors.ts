/** Typed errors: every rejection of the SDK is a {@link WebSignError}. */

import type { ErrorCode, ErrorDetails } from "./generated";

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
