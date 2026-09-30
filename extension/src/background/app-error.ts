/** A failure to reach or talk to the app, already classified with a protocol code. */

import type { ErrorCode, ErrorDetails } from "../generated";

export class AppError extends Error {
  constructor(
    readonly code: ErrorCode,
    message: string,
    readonly details?: ErrorDetails,
  ) {
    super(message);
    this.name = "AppError";
  }
}

/**
 * The code for a port that closed: a host that never spoke was not found
 * (`AppMissing`), one that did spoke and then vanished is a fault (`Internal`).
 */
export function closedCode(heard: boolean): ErrorCode {
  return heard ? "Internal" : "AppMissing";
}
