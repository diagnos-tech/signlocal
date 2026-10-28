// Browser-side helpers for the test page: pull the public key out of a certificate and check a
// signature with WebCrypto. Pure functions with no DOM access, so they are unit-tested in Bun
// with signatures made by OpenSSL (tests/unit/crypto.test.ts).

const OID_RSA = "1.2.840.113549.1.1.1";
const OID_EC = "1.2.840.10045.2.1";
const CURVES: Readonly<Record<string, "P-256" | "P-384" | "P-521">> = {
  "1.2.840.10045.3.1.7": "P-256",
  "1.3.132.0.34": "P-384",
  "1.3.132.0.35": "P-521",
};
const HASH_BYTES: Readonly<Record<string, number>> = {
  "SHA-256": 32,
  "SHA-384": 48,
  "SHA-512": 64,
};

interface Element {
  readonly tag: number;
  readonly pos: number;
  readonly start: number;
  readonly end: number;
}

/** Reads one DER element at `pos`; null when it does not fit in `bytes`. */
function readElement(bytes: Uint8Array, pos: number): Element | null {
  if (pos + 2 > bytes.length) return null;
  const tag = bytes[pos] ?? 0;
  let length = bytes[pos + 1] ?? 0;
  let start = pos + 2;
  if (length & 0x80) {
    const count = length & 0x7f;
    if (count < 1 || count > 4 || start + count > bytes.length) return null;
    length = 0;
    for (let i = 0; i < count; i++) length = length * 256 + (bytes[start + i] ?? 0);
    start += count;
  }
  const end = start + length;
  return end > bytes.length ? null : { tag, pos, start, end };
}

function children(bytes: Uint8Array, element: Element): Element[] | null {
  const list: Element[] = [];
  for (let pos = element.start; pos < element.end; ) {
    const child = readElement(bytes, pos);
    if (!child || child.end > element.end) return null;
    list.push(child);
    pos = child.end;
  }
  return list;
}

function decodeOid(bytes: Uint8Array, element: Element): string {
  const parts: number[] = [];
  let value = 0;
  for (let i = element.start; i < element.end; i++) {
    const byte = bytes[i] ?? 0;
    value = value * 128 + (byte & 0x7f);
    if (!(byte & 0x80)) {
      parts.push(value);
      value = 0;
    }
  }
  const first = parts.shift();
  if (first === undefined) return "";
  const head = first < 80 ? [Math.floor(first / 40), first % 40] : [2, first - 80];
  return head.concat(parts).join(".");
}

export type PublicKey =
  | { readonly type: "RSA"; readonly spki: Uint8Array }
  | { readonly type: "EC"; readonly curve: "P-256" | "P-384" | "P-521"; readonly spki: Uint8Array };

/**
 * Finds the SubjectPublicKeyInfo of an X.509 certificate: `{ type: "RSA", spki }` or
 * `{ type: "EC", curve, spki }` (spki is the full DER element), or null when the certificate
 * cannot be parsed or its key is of a kind WebCrypto does not handle.
 */
export function extractPublicKey(der: Uint8Array | ArrayBuffer): PublicKey | null {
  try {
    const bytes = der instanceof Uint8Array ? der : new Uint8Array(der);
    const certificate = readElement(bytes, 0);
    if (certificate?.tag !== 0x30 || certificate.end !== bytes.length) return null;
    const tbs = children(bytes, certificate)?.[0];
    if (tbs?.tag !== 0x30) return null;
    const fields = children(bytes, tbs);
    const first = fields?.[0];
    if (!fields || !first) return null;
    const spki = fields[(first.tag === 0xa0 ? 1 : 0) + 5];
    if (spki?.tag !== 0x30) return null;
    const [algorithm, key] = children(bytes, spki) ?? [];
    if (algorithm?.tag !== 0x30 || !key || key.tag !== 0x03) return null;
    const [oid, parameters] = children(bytes, algorithm) ?? [];
    if (oid?.tag !== 0x06) return null;
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

function importParameters(algorithm: string, key: PublicKey, hash: string) {
  if (algorithm === "ECDSA" && key.type === "EC") return { name: "ECDSA", namedCurve: key.curve };
  if (algorithm === "RSASSA-PKCS1-v1_5" && key.type === "RSA") return { name: algorithm, hash };
  if (algorithm === "RSASSA-PSS" && key.type === "RSA") return { name: "RSA-PSS", hash };
  return null;
}

export type Check = "verified" | "invalid" | "unsupported";

export interface SignatureToCheck {
  readonly certificateDer: Uint8Array | ArrayBuffer;
  readonly algorithm: string;
  readonly hash: string;
  readonly signature: Uint8Array;
  readonly message: Uint8Array;
}

/**
 * Checks `signature` over `message` the way WebCrypto does: the hash function digests the
 * message itself, so the digest the app signed must have been computed from exactly these bytes
 * with the same hash. "verified", "invalid" (the math says no) or "unsupported" (the key,
 * algorithm or browser cannot be checked here). Never throws.
 */
export async function verifySignature(
  subtle: SubtleCrypto,
  input: SignatureToCheck,
): Promise<Check> {
  const { algorithm, hash, signature, message } = input;
  const key = extractPublicKey(input.certificateDer);
  const params = key && HASH_BYTES[hash] ? importParameters(algorithm, key, hash) : null;
  if (!key || !params) return "unsupported";
  let publicKey: CryptoKey;
  try {
    publicKey = await subtle.importKey("spki", key.spki as BufferSource, params, false, ["verify"]);
  } catch {
    return "unsupported";
  }
  const verifyParams =
    algorithm === "ECDSA"
      ? { name: "ECDSA", hash }
      : algorithm === "RSASSA-PSS"
        ? { name: "RSA-PSS", saltLength: HASH_BYTES[hash] ?? 32 }
        : { name: "RSASSA-PKCS1-v1_5" };
  try {
    const ok = await subtle.verify(
      verifyParams,
      publicKey,
      signature as BufferSource,
      message as BufferSource,
    );
    return ok ? "verified" : "invalid";
  } catch {
    return "unsupported";
  }
}

/** The digest of `message` as bytes. */
export async function digestOf(
  subtle: SubtleCrypto,
  hash: string,
  message: Uint8Array,
): Promise<Uint8Array<ArrayBuffer>> {
  return new Uint8Array(await subtle.digest(hash, message as BufferSource));
}
