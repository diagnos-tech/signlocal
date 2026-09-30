/**
 * The few DER encodings a self-signed X.509 certificate needs (ITU-T X.690).
 * Encoding only: the fake never parses certificates, it builds them.
 */

/** Concatenates byte arrays. */
export function concat(...parts: readonly Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((sum, part) => sum + part.length, 0));
  let offset = 0;
  for (const part of parts) {
    out.set(part, offset);
    offset += part.length;
  }
  return out;
}

/** Tag, definite length, value. */
export function tlv(tag: number, ...content: readonly Uint8Array[]): Uint8Array {
  const value = concat(...content);
  const n = value.length;
  const length =
    n < 0x80
      ? Uint8Array.of(n)
      : n < 0x100
        ? Uint8Array.of(0x81, n)
        : n < 0x10000
          ? Uint8Array.of(0x82, n >> 8, n & 0xff)
          : Uint8Array.of(0x83, n >> 16, (n >> 8) & 0xff, n & 0xff);
  return concat(Uint8Array.of(tag), length, value);
}

export const sequence = (...items: readonly Uint8Array[]) => tlv(0x30, ...items);
export const set = (...items: readonly Uint8Array[]) => tlv(0x31, ...items);
export const nullValue = () => Uint8Array.of(0x05, 0x00);
export const octetString = (bytes: Uint8Array) => tlv(0x04, bytes);
export const utf8String = (text: string) => tlv(0x0c, new TextEncoder().encode(text));
/** A BIT STRING of whole bytes, or with `unused` trailing bits of the last byte. */
export const bitString = (bytes: Uint8Array, unused = 0) => tlv(0x03, Uint8Array.of(unused), bytes);
/** `[n] EXPLICIT`, constructed context-specific. */
export const explicit = (n: number, inner: Uint8Array) => tlv(0xa0 + n, inner);
export const boolean = (value: boolean) => Uint8Array.of(0x01, 0x01, value ? 0xff : 0x00);

/** A non-negative INTEGER from big-endian bytes (minimal, with a sign byte when needed). */
export function unsignedInteger(bytes: Uint8Array): Uint8Array {
  let start = 0;
  while (start < bytes.length - 1 && bytes[start] === 0) start++;
  const trimmed = bytes.subarray(start);
  const needsSign = ((trimmed[0] ?? 0) & 0x80) !== 0;
  return tlv(0x02, needsSign ? Uint8Array.of(0) : new Uint8Array(), trimmed);
}

/** An OBJECT IDENTIFIER from its dotted form. */
export function oid(dotted: string): Uint8Array {
  const [first = 0, second = 0, ...rest] = dotted.split(".").map(Number);
  const out: number[] = [];
  for (const arc of [first * 40 + second, ...rest]) {
    const groups = [arc & 0x7f];
    for (let value = Math.floor(arc / 128); value > 0; value = Math.floor(value / 128)) {
      groups.unshift((value & 0x7f) | 0x80);
    }
    out.push(...groups);
  }
  return tlv(0x06, Uint8Array.from(out));
}

/** X.509 Time: UTCTime through 2049, GeneralizedTime from 2050 (RFC 5280 §4.1.2.5). */
export function time(date: Date): Uint8Array {
  const iso = date.toISOString(); // 2025-01-01T00:00:00.000Z
  const digits = iso.slice(0, 19).replace(/[-T:]/g, "");
  const year = date.getUTCFullYear();
  const text = year < 2050 ? `${digits.slice(2)}Z` : `${digits}Z`;
  return tlv(year < 2050 ? 0x17 : 0x18, new TextEncoder().encode(text));
}
