import type { ErrorCode, ErrorDetails } from "./generated";

/** A failed call; `code` is the protocol's stable code. */
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
