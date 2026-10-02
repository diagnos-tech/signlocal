/** The `hello` negotiation: which protocol version both sides speak. */

import { WebSignError } from "./errors.js";
import type { AppEnvelope } from "./generated/index.js";

/** Newest protocol this library speaks; `hello` offers `{min: 1, max: PROTOCOL}`. */
export const PROTOCOL = 1;

/** Who is calling, for the app's logs. */
export interface ClientIdentity {
  readonly name: string;
  readonly version: string;
}

/** The `hello` request fields. */
export function helloFields(client: ClientIdentity): Record<string, unknown> {
  return { client, protocols: { min: 1, max: PROTOCOL } };
}

/**
 * The version the app chose. A choice outside our range means this library
 * is older than the app requires (`ClientOutdated`); the app answers with an
 * `error` itself when it is the older side.
 */
export function negotiatedVersion(reply: AppEnvelope): number {
  if (reply.type !== "hello" || !Number.isInteger(reply.protocol)) {
    throw new WebSignError("Internal", "the app's first message was not a hello reply");
  }
  if (reply.protocol < 1 || reply.protocol > PROTOCOL) {
    throw new WebSignError(
      "ClientOutdated",
      `the app chose protocol ${reply.protocol}; this library speaks 1 to ${PROTOCOL}`,
      { installed: String(PROTOCOL), required: String(reply.protocol) },
    );
  }
  return reply.protocol;
}
