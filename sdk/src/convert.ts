/** Wire ↔ public conversions: Base64 ↔ bytes, Unix seconds ↔ Date. */

import { malformedReply } from "./errors.js";
import type { Certificate as WireCertificate } from "./generated/index.js";
import type { Certificate } from "./types.js";

const ALPHABET = "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

/**
 * Strict padded standard Base64 → bytes. Anything a second encoder could
 * spell differently (whitespace, URL-safe alphabet, missing padding, stray
 * trailing bits) is refused, exactly like the protocol crate, so a value can
 * never decode two ways. Written without `atob` so it also runs in workers.
 * Only replies are decoded, so a failure is the extension's or app's bug.
 *
 * @throws {WebSignError} `Internal` on anything but canonical Base64.
 */
export function fromBase64(text: string): Uint8Array {
  const bad = () => malformedReply("malformed Base64");
  if (typeof text !== "string" || text.length % 4 !== 0) throw bad();
  const pad = text.endsWith("==") ? 2 : text.endsWith("=") ? 1 : 0;
  const bytes = new Uint8Array((text.length / 4) * 3 - pad);
  let acc = 0;
  let bits = 0;
  let out = 0;
  for (let i = 0; i < text.length - pad; i++) {
    const value = ALPHABET.indexOf(text.charAt(i));
    if (value < 0) throw bad();
    acc = (acc << 6) | value;
    bits += 6;
    if (bits >= 8) {
      bits -= 8;
      bytes[out++] = (acc >> bits) & 0xff;
    }
  }
  if ((acc & ((1 << bits) - 1)) !== 0) throw bad();
  return bytes;
}

/** Bytes → padded standard Base64. */
export function toBase64(bytes: Uint8Array): string {
  let text = "";
  for (let i = 0; i < bytes.length; i += 3) {
    const a = bytes[i] ?? 0;
    const b = bytes[i + 1] ?? 0;
    const c = bytes[i + 2] ?? 0;
    const group = (a << 16) | (b << 8) | c;
    text += ALPHABET.charAt(group >> 18) + ALPHABET.charAt((group >> 12) & 63);
    text += i + 1 < bytes.length ? ALPHABET.charAt((group >> 6) & 63) : "=";
    text += i + 2 < bytes.length ? ALPHABET.charAt(group & 63) : "=";
  }
  return text;
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
