// Browser-side helpers for the test-signature page: pull the public key out of
// a certificate and check a signature with WebCrypto. Pure functions with no
// DOM access so they can be unit-tested in Node and Bun (test/crypto.test.mjs).

const OID_RSA = "1.2.840.113549.1.1.1";
const OID_EC = "1.2.840.10045.2.1";
const CURVES = {
  "1.2.840.10045.3.1.7": "P-256",
  "1.3.132.0.34": "P-384",
  "1.3.132.0.35": "P-521",
};
const HASH_BYTES = { "SHA-256": 32, "SHA-384": 48, "SHA-512": 64 };

/** Reads one DER element at `pos`; returns null when it does not fit in `bytes`. */
function readElement(bytes, pos) {
  if (pos + 2 > bytes.length) return null;
  const tag = bytes[pos];
  let length = bytes[pos + 1];
  let start = pos + 2;
  if (length & 0x80) {
    const count = length & 0x7f;
    if (count < 1 || count > 4 || start + count > bytes.length) return null;
    length = 0;
    for (let i = 0; i < count; i++) length = length * 256 + bytes[start + i];
    start += count;
  }
  const end = start + length;
  if (end > bytes.length) return null;
  return { tag, start, end, pos };
}

function children(bytes, element) {
  const list = [];
  for (let pos = element.start; pos < element.end; ) {
    const child = readElement(bytes, pos);
    if (!child || child.end > element.end) return null;
    list.push(child);
    pos = child.end;
  }
  return list;
}

function decodeOid(bytes, element) {
  const parts = [];
  let value = 0;
  for (let i = element.start; i < element.end; i++) {
    value = value * 128 + (bytes[i] & 0x7f);
    if (!(bytes[i] & 0x80)) {
      parts.push(value);
      value = 0;
    }
  }
  if (parts.length === 0) return "";
  const first = parts.shift();
  const head = first < 80 ? [Math.floor(first / 40), first % 40] : [2, first - 80];
  return head.concat(parts).join(".");
}

/**
 * Finds the SubjectPublicKeyInfo of an X.509 certificate.
 * Returns `{ type: "RSA", spki }` or `{ type: "EC", curve, spki }` (spki is the
 * full DER element), or null when the certificate cannot be parsed or its key
 * is of a kind WebCrypto does not handle.
 */
export function extractPublicKey(der) {
  try {
    const bytes = der instanceof Uint8Array ? der : new Uint8Array(der);
    const certificate = readElement(bytes, 0);
    if (!certificate || certificate.tag !== 0x30 || certificate.end !== bytes.length) return null;
    const tbs = children(bytes, certificate)?.[0];
    if (!tbs || tbs.tag !== 0x30) return null;
    const fields = children(bytes, tbs);
    if (!fields) return null;
    const offset = fields[0].tag === 0xa0 ? 1 : 0;
    const spki = fields[offset + 5];
    if (!spki || spki.tag !== 0x30) return null;
    const [algorithm, key] = children(bytes, spki) ?? [];
    if (!algorithm || algorithm.tag !== 0x30 || !key || key.tag !== 0x03) return null;
    const [oid, parameters] = children(bytes, algorithm) ?? [];
    if (!oid || oid.tag !== 0x06) return null;
    const copy = bytes.slice(spki.pos, spki.end);
    const name = decodeOid(bytes, oid);
    if (name === OID_RSA) return { type: "RSA", spki: copy };
    if (name === OID_EC && parameters && parameters.tag === 0x06) {
      const curve = CURVES[decodeOid(bytes, parameters)];
      return curve ? { type: "EC", curve, spki: copy } : null;
    }
    return null;
  } catch {
    return null;
  }
}

function importParameters(algorithm, key, hash) {
  if (algorithm === "ECDSA" && key.type === "EC") return { name: "ECDSA", namedCurve: key.curve };
  if (algorithm === "RSASSA-PKCS1-v1_5" && key.type === "RSA") return { name: algorithm, hash };
  if (algorithm === "RSASSA-PSS" && key.type === "RSA") return { name: "RSA-PSS", hash };
  return null;
}

/**
 * Checks `signature` over `message` the way WebCrypto does: the hash function
 * digests the message itself, so the digest the app signed must have been
 * computed from exactly these bytes with the same hash.
 *
 * Returns "verified", "invalid" (the math says no) or "unsupported" (the key,
 * algorithm or browser cannot be checked here). Never throws.
 */
export async function verifySignature(subtle, { certificateDer, algorithm, hash, signature, message }) {
  const key = extractPublicKey(certificateDer);
  const params = key && HASH_BYTES[hash] ? importParameters(algorithm, key, hash) : null;
  if (!params) return "unsupported";
  let publicKey;
  try {
    publicKey = await subtle.importKey("spki", key.spki, params, false, ["verify"]);
  } catch {
    return "unsupported";
  }
  const verifyParams =
    algorithm === "ECDSA"
      ? { name: "ECDSA", hash }
      : algorithm === "RSASSA-PSS"
        ? { name: "RSA-PSS", saltLength: HASH_BYTES[hash] }
        : { name: "RSASSA-PKCS1-v1_5" };
  try {
    return (await subtle.verify(verifyParams, publicKey, signature, message)) ? "verified" : "invalid";
  } catch {
    return "unsupported";
  }
}

/** The digest of `message` as bytes. */
export async function digestOf(subtle, hash, message) {
  return new Uint8Array(await subtle.digest(hash, message));
}
