/** BigInt helpers for the fake's signing math (RSA and P-256). */

/** Big-endian unsigned bytes → bigint. */
export function toBigInt(bytes: Uint8Array): bigint {
  let value = 0n;
  for (const byte of bytes) value = (value << 8n) | BigInt(byte);
  return value;
}

/** bigint → big-endian unsigned bytes, left-padded to `length`. */
export function toBytes(value: bigint, length: number): Uint8Array {
  const out = new Uint8Array(length);
  let rest = value;
  for (let i = length - 1; i >= 0; i--) {
    out[i] = Number(rest & 0xffn);
    rest >>= 8n;
  }
  return out;
}

/** The non-negative remainder. */
export const mod = (a: bigint, m: bigint): bigint => ((a % m) + m) % m;

/** `base^exponent mod m` by square-and-multiply. */
export function modPow(base: bigint, exponent: bigint, m: bigint): bigint {
  let result = 1n;
  let b = mod(base, m);
  for (let e = exponent; e > 0n; e >>= 1n) {
    if (e & 1n) result = (result * b) % m;
    b = (b * b) % m;
  }
  return result;
}

/** The inverse of `a` modulo the prime `p` (Fermat). */
export const modInverse = (a: bigint, p: bigint): bigint => modPow(a, p - 2n, p);

/** Base64url (JWK) → bytes. */
export function fromBase64Url(text: string): Uint8Array {
  const standard = text.replace(/-/g, "+").replace(/_/g, "/");
  const binary = atob(standard.padEnd(Math.ceil(standard.length / 4) * 4, "="));
  return Uint8Array.from(binary, (c) => c.charCodeAt(0));
}
