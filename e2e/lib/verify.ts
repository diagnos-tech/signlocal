/**
 * The independent verifier: every signature the suite receives is checked
 * with node:crypto against the certificate's public key, never by the code
 * under test. The digest the page hands over is the hash of a message the
 * test chose, so the check is an ordinary verification of that message.
 */

import { constants, verify, type X509Certificate } from "node:crypto";

export type HashName = "SHA-256" | "SHA-384" | "SHA-512";
export type SignatureAlgorithm = "ECDSA" | "RSASSA-PKCS1-v1_5" | "RSASSA-PSS";

const DIGEST_BYTES: Record<HashName, number> = { "SHA-256": 32, "SHA-384": 48, "SHA-512": 64 };

/**
 * Whether `signature` (RSA block, or ECDSA r‖s as the SDK returns it) signs
 * `message` hashed with `hash`, under the key of `certificate`.
 */
export function verifies(
  certificate: X509Certificate,
  hash: HashName,
  algorithm: SignatureAlgorithm,
  message: Uint8Array,
  signature: Uint8Array,
): boolean {
  const name = hash.replace("-", "").toLowerCase();
  const key = certificate.publicKey;
  switch (algorithm) {
    case "ECDSA":
      return verify(name, message, { key, dsaEncoding: "ieee-p1363" }, signature);
    case "RSASSA-PKCS1-v1_5":
      return verify(name, message, { key, padding: constants.RSA_PKCS1_PADDING }, signature);
    case "RSASSA-PSS":
      // The protocol's PSS: MGF1 with the same hash, salt as long as the digest.
      return verify(
        name,
        message,
        { key, padding: constants.RSA_PKCS1_PSS_PADDING, saltLength: DIGEST_BYTES[hash] },
        signature,
      );
  }
}
