/**
 * ECDSA over P-256 on a precomputed digest. WebCrypto only signs messages (it
 * hashes them itself), while the protocol hands the signer a digest, so the
 * fake does the curve math with BigInt. The nonce follows RFC 6979
 * (HMAC-SHA-256), which makes every signature deterministic.
 */

import { mod, modInverse, toBigInt, toBytes } from "./bigint.js";
import { concat } from "./der.js";

const P = 0xffffffff00000001000000000000000000000000ffffffffffffffffffffffffn;
const N = 0xffffffff00000000ffffffffffffffffbce6faada7179e84f3b9cac2fc632551n;
const G: Affine = [
  0x6b17d1f2e12c4247f8bce6e563a440f277037d812deb33a0f4a13945d898c296n,
  0x4fe342e2fe1a7f9b8ee7eb4a7c0f9e162bce33576b315ececbb6406837bf51f5n,
];

type Affine = readonly [bigint, bigint];
/** Jacobian coordinates (X/Z², Y/Z³); Z = 0 is the point at infinity. */
type Jacobian = readonly [bigint, bigint, bigint];

const INFINITY: Jacobian = [1n, 1n, 0n];

function double([x, y, z]: Jacobian): Jacobian {
  if (z === 0n || y === 0n) return INFINITY;
  // a = -3: M = 3 (X - Z²)(X + Z²).
  const z2 = mod(z * z, P);
  const m = mod(3n * (x - z2) * (x + z2), P);
  const y2 = mod(y * y, P);
  const s = mod(4n * x * y2, P);
  const x3 = mod(m * m - 2n * s, P);
  return [x3, mod(m * (s - x3) - 8n * y2 * y2, P), mod(2n * y * z, P)];
}

function add(a: Jacobian, b: Jacobian): Jacobian {
  if (a[2] === 0n) return b;
  if (b[2] === 0n) return a;
  const [x1, y1, z1] = a;
  const [x2, y2, z2] = b;
  const z1z1 = mod(z1 * z1, P);
  const z2z2 = mod(z2 * z2, P);
  const u1 = mod(x1 * z2z2, P);
  const u2 = mod(x2 * z1z1, P);
  const s1 = mod(y1 * z2 * z2z2, P);
  const s2 = mod(y2 * z1 * z1z1, P);
  if (u1 === u2) return s1 === s2 ? double(a) : INFINITY;
  const h = mod(u2 - u1, P);
  const r = mod(s2 - s1, P);
  const h2 = mod(h * h, P);
  const h3 = mod(h * h2, P);
  const x3 = mod(r * r - h3 - 2n * u1 * h2, P);
  return [x3, mod(r * (u1 * h2 - x3) - s1 * h3, P), mod(h * z1 * z2, P)];
}

/** The x coordinate of `k·G`. */
function baseMultiplyX(k: bigint): bigint {
  let result = INFINITY;
  let addend: Jacobian = [G[0], G[1], 1n];
  for (let e = k; e > 0n; e >>= 1n) {
    if (e & 1n) result = add(result, addend);
    addend = double(addend);
  }
  const zInverse = modInverse(result[2], P);
  return mod(result[0] * zInverse * zInverse, P);
}

async function hmac(key: Uint8Array, ...parts: Uint8Array[]): Promise<Uint8Array> {
  const cryptoKey = await crypto.subtle.importKey(
    "raw",
    key as BufferSource,
    { name: "HMAC", hash: "SHA-256" },
    false,
    ["sign"],
  );
  return new Uint8Array(
    await crypto.subtle.sign("HMAC", cryptoKey, concat(...parts) as BufferSource),
  );
}

/** The leftmost 256 bits of `bytes` as an integer (ECDSA truncates longer digests). */
const bits2int = (bytes: Uint8Array): bigint => toBigInt(bytes.subarray(0, 32));

/**
 * Signs `digest` with the private scalar `d` (32 bytes); returns raw r‖s,
 * 64 bytes (IEEE P1363), the shape the protocol and WebCrypto use.
 */
export async function signP256(d: Uint8Array, digest: Uint8Array): Promise<Uint8Array> {
  const secret = toBigInt(d);
  const z = bits2int(digest);
  const x = toBytes(secret, 32);
  const h1 = toBytes(mod(z, N), 32);
  // RFC 6979 §3.2 steps b–h.
  let v: Uint8Array = new Uint8Array(32).fill(1);
  let k: Uint8Array = new Uint8Array(32);
  k = await hmac(k, v, Uint8Array.of(0), x, h1);
  v = await hmac(k, v);
  k = await hmac(k, v, Uint8Array.of(1), x, h1);
  v = await hmac(k, v);
  for (;;) {
    v = await hmac(k, v);
    const nonce = toBigInt(v);
    if (nonce >= 1n && nonce < N) {
      const r = mod(baseMultiplyX(nonce), N);
      const s = mod(modInverse(nonce, N) * (z + r * secret), N);
      if (r !== 0n && s !== 0n) return concat(toBytes(r, 32), toBytes(s, 32));
    }
    k = await hmac(k, v, Uint8Array.of(0));
    v = await hmac(k, v);
  }
}
