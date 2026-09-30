/** Turns the wire certificate (Base64, Unix seconds) into the public one (bytes, `Date`). */

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
  if (typeof wire !== "object" || wire === null) throw malformed("malformed certificate");
  const { der, chain, notBefore, notAfter, ...rest } = wire as WireCertificate;
  if (!Array.isArray(chain)) throw malformed("certificate without a chain list");
  return {
    ...rest,
    der: decodeBase64(der),
    chain: chain.map(decodeBase64),
    notBefore: toDate(notBefore),
    notAfter: toDate(notAfter),
  };
}

function toDate(unixSeconds: unknown): Date {
  const date = new Date(Number(unixSeconds) * 1000);
  if (typeof unixSeconds !== "number" || Number.isNaN(date.getTime())) {
    throw malformed("certificate with an invalid validity date");
  }
  return date;
}

function malformed(what: string): WebSignError {
  return new WebSignError("Internal", `the app sent a ${what}`);
}
