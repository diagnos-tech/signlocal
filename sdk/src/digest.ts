/** The digest step of `sign()`: what the app asks for and what `prepare` returns. */

import { viewOf } from "./convert.js";
import { WebSignError } from "./errors.js";
import type { HashName, NeedDigest, SignatureAlgorithmName } from "./generated/index.js";
import { ALGORITHMS, DIGEST_LENGTH } from "./validate.js";

/**
 * What `prepare` returned, as bytes of the exact length of `hash`. Checked
 * here rather than by the app so a wrong length (a SHA-1 digest, a hex
 * string) fails with a message that names the mistake, before anything
 * reaches the person's screen.
 *
 * @throws {WebSignError} `InvalidRequest` for anything but a right-sized `Uint8Array`/`ArrayBuffer`.
 */
export function toDigestBytes(value: unknown, hash: HashName): Uint8Array {
  const bytes = viewOf(value);
  if (bytes === undefined) {
    throw new WebSignError(
      "InvalidRequest",
      "prepare() must return a Uint8Array or an ArrayBuffer.",
    );
  }
  const required = DIGEST_LENGTH[hash];
  if (bytes.length !== required) {
    throw new WebSignError(
      "InvalidRequest",
      `Digest is ${bytes.length} bytes; ${hash} requires ${required}.`,
    );
  }
  return bytes;
}

/**
 * Refuses a `need_digest` that contradicts the request before `prepare`
 * runs: a different hash or an algorithm the site did not accept would make
 * the site build signed attributes for a signature it never asked for.
 * `InvalidRequest` is the protocol's code for a broken signing sequence.
 *
 * @throws {WebSignError} `InvalidRequest`.
 */
export function checkNeed(
  need: NeedDigest,
  hash: HashName,
  algorithms: readonly SignatureAlgorithmName[] | undefined,
): void {
  if (need.hash !== hash) {
    throw new WebSignError(
      "InvalidRequest",
      `The app asked for ${need.hash}; sign() asked for ${hash}.`,
    );
  }
  if (!(algorithms ?? ALGORITHMS).includes(need.algorithm)) {
    throw new WebSignError(
      "InvalidRequest",
      `The app chose ${need.algorithm}, which sign() did not accept.`,
    );
  }
}
