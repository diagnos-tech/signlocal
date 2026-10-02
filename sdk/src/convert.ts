/** Wire ↔ public conversions: Base64 ↔ bytes, Unix seconds ↔ Date. */

import { malformedReply } from "./errors.js";
import type { Certificate as WireCertificate } from "./generated/index.js";
import type { Certificate } from "./types.js";

/**
 * Strict padded standard Base64 → bytes. Anything a second encoder could
 * spell differently (whitespace, URL-safe alphabet, missing padding, stray
 * trailing bits) is refused, exactly like the protocol crate, so a value can
 * never decode two ways: the text must be what encoding its bytes gives back.
 * Only replies are decoded, so a failure is the extension's or app's bug.
 *
 * @throws {WebSignError} `Internal` on anything but canonical Base64.
 */
export function fromBase64(text: string): Uint8Array<ArrayBuffer> {
  let binary: string | undefined;
  try {
    binary = atob(text);
  } catch {
    // Not Base64 at all; refused below.
  }
  if (typeof text !== "string" || binary === undefined || btoa(binary) !== text) {
    throw malformedReply("malformed Base64");
  }
  return Uint8Array.from(binary, (c) => c.charCodeAt(0));
}

/** Bytes → padded standard Base64. */
export function toBase64(bytes: Uint8Array): string {
  let binary = "";
  for (const byte of bytes) binary += String.fromCharCode(byte);
  return btoa(binary);
}

/** A view of exactly the bytes of `value`, or undefined when it holds no bytes. */
export function viewOf(value: unknown): Uint8Array | undefined {
  if (ArrayBuffer.isView(value))
    return new Uint8Array(value.buffer, value.byteOffset, value.byteLength);
  return value instanceof ArrayBuffer ? new Uint8Array(value) : undefined;
}

/** Wire certificate → public certificate: bytes decoded, Unix seconds → `Date`, the rest copied. */
export function toCertificate(wire: WireCertificate): Certificate {
  const { icpBrasil, eidas, keyStorage } = wire.profile;
  return {
    der: fromBase64(wire.der),
    chain: wire.chain.map(fromBase64),
    fingerprint: wire.fingerprint,
    displayName: wire.displayName,
    issuerName: wire.issuerName,
    notBefore: new Date(wire.notBefore * 1000),
    notAfter: new Date(wire.notAfter * 1000),
    key: { ...wire.key },
    algorithms: [...wire.algorithms],
    profile: {
      ...(icpBrasil === undefined ? {} : { icpBrasil }),
      ...(eidas === undefined ? {} : { eidas: { ...eidas, types: [...eidas.types] } }),
      keyStorage,
    },
  };
}
