/** Checks of what the caller passed, before anything is sent. */

import { WebSignError } from "./errors.js";
import type { HashName, SignatureAlgorithmName } from "./generated/index.js";

const HASHES: readonly string[] = ["SHA-256", "SHA-384", "SHA-512"];
/** Every algorithm the SDK knows. */
export const ALGORITHMS: readonly string[] = ["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"];

/** Digest bytes for each hash. */
export const DIGEST_LENGTH: Readonly<Record<HashName, number>> = {
  "SHA-256": 32,
  "SHA-384": 48,
  "SHA-512": 64,
};

/** @throws {WebSignError} `InvalidRequest` unless `hash` is SHA-256, SHA-384 or SHA-512. */
export function checkHash(hash: unknown): HashName {
  if (typeof hash === "string" && HASHES.includes(hash)) return hash as HashName;
  throw new WebSignError(
    "InvalidRequest",
    `Unknown hash ${JSON.stringify(hash)}; use "SHA-256", "SHA-384" or "SHA-512".`,
  );
}

/**
 * One algorithm or a list → a deduplicated list in the caller's order;
 * `undefined` when none was given.
 *
 * @throws {WebSignError} `InvalidRequest` for unknown or empty input (an empty
 * list is almost always a bug upstream, and silently meaning "any" would
 * widen what the person is asked to approve).
 */
export function checkAlgorithms(input: unknown): SignatureAlgorithmName[] | undefined {
  if (input === undefined) return undefined;
  const list: readonly unknown[] = Array.isArray(input) ? input : [input];
  const bad = list.find((a) => typeof a !== "string" || !ALGORITHMS.includes(a));
  if (list.length === 0 || bad !== undefined) {
    throw new WebSignError(
      "InvalidRequest",
      `Unknown algorithm ${JSON.stringify(bad ?? input)}; use "ECDSA", "RSASSA-PKCS1-v1_5" or "RSASSA-PSS".`,
    );
  }
  return [...new Set(list as SignatureAlgorithmName[])];
}

const FINGERPRINT = /^[0-9a-f]{64}$/;

/**
 * The fingerprint of a preselected certificate, exactly as `certificates()`
 * returned it (lowercase). Only the fingerprint is sent: the certificate
 * body never leaves the page.
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
    "certificate must be a Certificate or its fingerprint (64 lowercase hex digits).",
  );
}
