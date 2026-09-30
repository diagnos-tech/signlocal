/**
 * What every spec shares: the environment, the keys and the checks of a
 * signature reply.
 */

import { X509Certificate } from "node:crypto";

import { expect } from "@playwright/test";

import { environment } from "./environment.ts";
import type { SignReply } from "./fixture.ts";
import { signingKeys, type TestKey, testKeys } from "./keys.ts";
import { type HashName, type SignatureAlgorithm, verifies } from "./verify.ts";

export const env = environment();
export const keys: readonly TestKey[] = testKeys(env.keys);
/** RSA and every NIST curve the key source has. */
export const signers: readonly TestKey[] = signingKeys(keys);
export const HASHES: readonly HashName[] = ["SHA-256", "SHA-384", "SHA-512"];

/** Why a window scenario is skipped without a display. */
export const NEEDS_WINDOW = "needs the confirmation window (WEBSIGN_E2E_WINDOW=headless)";

/** The first key of `type`; fails the test when the source has none. */
export function keyOf(type: "RSA" | "EC"): TestKey {
  const key = signers.find((k) => k.type === type);
  if (key === undefined) throw new Error(`no ${type} software key in this run`);
  return key;
}

/**
 * Asserts `reply` is a signature of `message` by `key`, verified with
 * node:crypto, and that `prepare` ran once, for that key.
 */
export function expectSigned(
  reply: SignReply,
  key: TestKey,
  hash: HashName,
  algorithm: SignatureAlgorithm,
  message: string,
): void {
  if (!reply.ok) throw new Error(`signing failed with ${reply.code}`);
  expect(reply.fingerprint).toBe(key.fingerprint);
  expect(reply.prepared).toEqual([{ fingerprint: key.fingerprint, algorithm }]);
  expect(reply.hash).toBe(hash);
  expect(reply.algorithm).toBe(algorithm);
  const certificate = new X509Certificate(Buffer.from(reply.certificate, "base64"));
  expect(certificate.fingerprint256).toBe(key.certificate.fingerprint256);
  const signature = Buffer.from(reply.signature, "base64");
  expect(verifies(certificate, hash, algorithm, Buffer.from(message, "hex"), signature)).toBe(true);
}
