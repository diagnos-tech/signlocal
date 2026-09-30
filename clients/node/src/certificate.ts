/** Turns the wire certificate (Base64 strings) into the public one (bytes). */

import { decodeBase64 } from "./convert.js";
import { WebSignError } from "./errors.js";
import type { Certificate as WireCertificate } from "./generated/index.js";
import type { Certificate } from "./types.js";

/**
 * Decodes `der` and `chain` once, at the boundary, so callers hand bytes
 * straight to their CMS/PAdES code instead of each re-decoding Base64. Strict:
 * a certificate the app garbled fails the request with `Internal` rather than
 * reaching a signature format half-decoded.
 */
export function decodeCertificate(wire: unknown): Certificate {
  if (typeof wire !== "object" || wire === null) {
    throw new WebSignError("Internal", "the app sent a malformed certificate");
  }
  const { der, chain, ...rest } = wire as WireCertificate;
  if (!Array.isArray(chain)) {
    throw new WebSignError("Internal", "the app sent a certificate without a chain list");
  }
  return { ...rest, der: decodeBase64(der), chain: chain.map(decodeBase64) };
}
