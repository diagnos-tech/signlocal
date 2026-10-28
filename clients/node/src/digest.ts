/** The digest step of `sign()`: what the app asks for and what `prepare` returns. */

import { SignlocalError } from "./errors.js";
import type { NeedDigest } from "./generated/index.js";
import type { HashAlgorithm, SignatureAlgorithm } from "./types.js";
import { ALGORITHMS, DIGEST_LENGTH } from "./validate.js";

/**
 * What `prepare` returned, as bytes of the exact length of `hash`. Checked
 * here rather than by the app so a wrong length (a SHA-1 digest, a hex
 * string) fails with a message that names the mistake.
 *
 * @throws {SignlocalError} `InvalidRequest` for anything but a right-sized `Uint8Array`/`ArrayBuffer`.
 */
export function toDigestBytes(value: unknown, hash: HashAlgorithm): Uint8Array {
  const bytes = ArrayBuffer.isView(value)
    ? new Uint8Array(value.buffer, value.byteOffset, value.byteLength)
    : value instanceof ArrayBuffer
      ? new Uint8Array(value)
      : undefined;
  if (bytes === undefined) {
    throw new SignlocalError(
      "InvalidRequest",
      "prepare must return a Uint8Array or an ArrayBuffer",
    );
  }
  const required = DIGEST_LENGTH[hash];
  if (bytes.length !== required) {
    throw new SignlocalError(
      "InvalidRequest",
      `digest is ${bytes.length} bytes; ${hash} requires ${required}`,
    );
  }
  return bytes;
}

/**
 * Refuses a `need_digest` that contradicts the request before `prepare`
 * runs: a different hash, or an algorithm the caller did not accept, would
 * make the caller build signed attributes for a signature it never asked
 * for. `InvalidRequest` is the protocol's code for a broken signing sequence.
 *
 * @throws {SignlocalError} `InvalidRequest`.
 */
export function checkNeed(
  need: Pick<NeedDigest, "hash" | "algorithm">,
  hash: HashAlgorithm,
  algorithms: readonly SignatureAlgorithm[] | undefined,
): void {
  if (need.hash !== hash) {
    throw new SignlocalError(
      "InvalidRequest",
      `the app asked for a ${String(need.hash)} digest; sign() asked for ${hash}`,
    );
  }
  if (!(algorithms ?? ALGORITHMS).includes(need.algorithm)) {
    throw new SignlocalError(
      "InvalidRequest",
      `the app chose ${String(need.algorithm)}, which sign() did not accept`,
    );
  }
}
