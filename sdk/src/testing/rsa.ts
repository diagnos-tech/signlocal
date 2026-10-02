/**
 * RSA signatures on a precomputed digest: EMSA-PKCS1-v1_5 and EMSA-PSS
 * (RFC 8017 §9), then the private-key operation with BigInt and the CRT
 * parameters of the JWK. WebCrypto cannot do this: it only signs messages.
 */

import type { HashAlgorithm, SignatureAlgorithm } from "../types.js";
import { fromBase64Url, mod, modPow, toBigInt, toBytes } from "./bigint.js";
import { concat } from "./der.js";

/** DER of DigestInfo up to the digest bytes (RFC 8017 §9.2 note 1). */
const DIGEST_INFO: Readonly<Record<HashAlgorithm, string>> = {
  "SHA-256": "3031300d060960864801650304020105000420",
  "SHA-384": "3041300d060960864801650304020205000430",
  "SHA-512": "3051300d060960864801650304020305000440",
};

const fromHex = (hex: string) =>
  Uint8Array.from(hex.match(/../g) ?? [], (byte) => Number.parseInt(byte, 16));

async function hash(name: HashAlgorithm, data: Uint8Array): Promise<Uint8Array> {
  return new Uint8Array(await crypto.subtle.digest(name, data as BufferSource));
}

function pkcs1(digest: Uint8Array, name: HashAlgorithm, length: number): Uint8Array {
  const t = concat(fromHex(DIGEST_INFO[name]), digest);
  const padding = new Uint8Array(length - t.length - 3).fill(0xff);
  return concat(Uint8Array.of(0, 1), padding, Uint8Array.of(0), t);
}

/** MGF1 with `name`, `length` bytes. */
async function mgf1(seed: Uint8Array, length: number, name: HashAlgorithm): Promise<Uint8Array> {
  const blocks: Uint8Array[] = [];
  for (let counter = 0, total = 0; total < length; counter++) {
    const block = await hash(name, concat(seed, toBytes(BigInt(counter), 4)));
    blocks.push(block);
    total += block.length;
  }
  return concat(...blocks).subarray(0, length);
}

/**
 * EMSA-PSS with a salt as long as the digest. The salt is derived from the
 * digest instead of random, so the fake's signatures are reproducible; any
 * salt verifies.
 */
async function pss(digest: Uint8Array, name: HashAlgorithm, modBits: number): Promise<Uint8Array> {
  const emLength = Math.ceil((modBits - 1) / 8);
  const salt = await hash(name, concat(new TextEncoder().encode("websign-fake-salt"), digest));
  const h = await hash(name, concat(new Uint8Array(8), digest, salt));
  const db = concat(new Uint8Array(emLength - salt.length - h.length - 2), Uint8Array.of(1), salt);
  const mask = await mgf1(h, db.length, name);
  const masked = db.map((byte, i) => byte ^ (mask[i] ?? 0));
  masked[0] = (masked[0] ?? 0) & (0xff >> (8 * emLength - (modBits - 1)));
  return concat(masked, h, Uint8Array.of(0xbc));
}

/** Signs `digest` with the RSA private JWK; the result is as long as the modulus. */
export async function signRsa(
  key: JsonWebKey,
  algorithm: SignatureAlgorithm,
  name: HashAlgorithm,
  digest: Uint8Array,
): Promise<Uint8Array> {
  const [n, p, q, dp, dq, qi] = [key.n, key.p, key.q, key.dp, key.dq, key.qi].map((part) =>
    toBigInt(fromBase64Url(part ?? "")),
  ) as [bigint, bigint, bigint, bigint, bigint, bigint];
  const modBits = n.toString(2).length;
  const length = Math.ceil(modBits / 8);
  const encoded =
    algorithm === "RSASSA-PSS" ? await pss(digest, name, modBits) : pkcs1(digest, name, length);
  const m = toBigInt(encoded);
  // CRT (RFC 8017 §5.1.2): two half-size exponentiations instead of one.
  const m1 = modPow(m, dp, p);
  const m2 = modPow(m, dq, q);
  const h = mod(qi * (m1 - m2), p);
  return toBytes(m2 + h * q, length);
}
