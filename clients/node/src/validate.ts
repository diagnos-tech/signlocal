/** Checks of what the caller passed, before anything is sent; same rules as `@websign/sdk`. */

import { WebSignError } from "./errors.js";
import type { HashAlgorithm, SignatureAlgorithm } from "./types.js";

const HASHES: readonly string[] = ["SHA-256", "SHA-384", "SHA-512"];
/** Every algorithm this library knows. */
export const ALGORITHMS: readonly SignatureAlgorithm[] = [
  "ECDSA",
  "RSASSA-PKCS1-v1_5",
  "RSASSA-PSS",
];

/** Digest bytes for each hash. */
export const DIGEST_LENGTH: Readonly<Record<HashAlgorithm, number>> = {
  "SHA-256": 32,
  "SHA-384": 48,
  "SHA-512": 64,
};

/** @throws {WebSignError} `InvalidRequest` unless `hash` is SHA-256, SHA-384 or SHA-512. */
export function checkHash(hash: unknown): HashAlgorithm {
  if (typeof hash === "string" && HASHES.includes(hash)) return hash as HashAlgorithm;
  throw new WebSignError("InvalidRequest", `unsupported hash: ${String(hash)}`);
}

/**
 * One algorithm or a list → a deduplicated list in the caller's order;
 * `undefined` when none was given.
 *
 * @throws {WebSignError} `InvalidRequest` for unknown names or an empty list:
 * an empty list is almost always a bug upstream, and silently meaning "any"
 * would widen what the person is asked to approve.
 */
export function checkAlgorithms(input: unknown): SignatureAlgorithm[] | undefined {
  if (input === undefined) return undefined;
  const list: readonly unknown[] = Array.isArray(input) ? input : [input];
  const known = (name: unknown) => ALGORITHMS.includes(name as SignatureAlgorithm);
  if (list.length === 0 || !list.every(known)) {
    throw new WebSignError(
      "InvalidRequest",
      "algorithm must be one of, or a non-empty list of, ECDSA, RSASSA-PKCS1-v1_5, RSASSA-PSS",
    );
  }
  return [...new Set(list as SignatureAlgorithm[])];
}

const FINGERPRINT = /^[0-9a-f]{64}$/;

/**
 * The fingerprint of a preselected certificate, given as a `Certificate`
 * from `certificates()` or as its fingerprint. Only the fingerprint is sent.
 *
 * @throws {WebSignError} `InvalidRequest` for anything else.
 */
export function checkFingerprint(certificate: unknown): string | undefined {
  if (certificate === undefined) return undefined;
  const fingerprint =
    typeof certificate === "object" && certificate !== null
      ? (certificate as { fingerprint?: unknown }).fingerprint
      : certificate;
  if (typeof fingerprint === "string" && FINGERPRINT.test(fingerprint)) return fingerprint;
  throw new WebSignError(
    "InvalidRequest",
    "certificate must be a Certificate from certificates() or its SHA-256 fingerprint (64 lowercase hex digits)",
  );
}
