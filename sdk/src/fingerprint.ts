import { viewOf } from "./convert.js";
import { WebSignError } from "./errors.js";
import type { Bytes, VerificationCode } from "./types.js";

/**
 * The verification code of `digest`, identical to the one the app shows
 * (same algorithm as websign-protocol `verification_code`). Show it next to
 * your own Sign button so people can compare what they are about to sign
 * with what the app displays.
 *
 * @example
 * const { text } = fingerprint(await crypto.subtle.digest("SHA-256", signedAttributes));
 * code.textContent = text; // "7F3A 9C21 E0B4 55D8"
 *
 * @throws {WebSignError} `InvalidRequest` when `digest` has fewer than 8 bytes.
 */
export function fingerprint(digest: Bytes): VerificationCode {
  const bytes = viewOf(digest) ?? new Uint8Array();
  if (bytes.length < 8) {
    throw new WebSignError(
      "InvalidRequest",
      `Digest is ${bytes.length} bytes; a verification code needs at least 8.`,
    );
  }
  const head = Array.from(bytes.subarray(0, 8));
  const bits = ((head[1] ?? 0) << 8) | (head[2] ?? 0);
  const hex = head.map((x) => x.toString(16).padStart(2, "0")).join("");
  const cells: boolean[] = [];
  for (let row = 0; row < 5; row++) {
    const [a, b, c] = [0, 1, 2].map((column) => ((bits >> (row * 3 + column)) & 1) === 1);
    cells.push(!!a, !!b, !!c, !!b, !!a);
  }
  return {
    text: hex.toUpperCase().replace(/(.{4})(?=.)/g, "$1 "),
    colorIndex: (head[0] ?? 0) >> 5,
    cells,
  };
}
