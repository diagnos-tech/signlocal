// Run: bun test site/tests   (or: node --test site/tests)
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { test } from "node:test";
import { digestOf, extractPublicKey, verifySignature } from "../assets/test-crypto.js";

const vectors = JSON.parse(readFileSync(new URL("./vectors.json", import.meta.url), "utf8"));
const subtle = globalThis.crypto.subtle;
const message = new TextEncoder().encode(vectors.message);
const bytes = (b64) => new Uint8Array(Buffer.from(b64, "base64"));

/** openssl writes ECDSA as DER SEQUENCE { r, s }; WebCrypto and the SDK use r||s. */
function derToRaw(der, size) {
  const read = (pos) => {
    let length = der[pos + 1];
    let start = pos + 2;
    if (length & 0x80) {
      const n = length & 0x7f;
      length = 0;
      for (let i = 0; i < n; i++) length = length * 256 + der[start + i];
      start += n;
    }
    return { start, end: start + length };
  };
  const seq = read(0);
  const r = read(seq.start);
  const s = read(r.end);
  const pad = (v) => {
    const raw = der.slice(v.start, v.end);
    const trimmed = raw.slice(raw.findIndex((x) => x !== 0));
    const out = new Uint8Array(size);
    out.set(trimmed, size - trimmed.length);
    return out;
  };
  return Uint8Array.from([...pad(r), ...pad(s)]);
}

const HASHES = { sha256: "SHA-256", sha384: "SHA-384", sha512: "SHA-512" };
const SIZES = { p256: 32, p384: 48, p521: 66 };

test("extractPublicKey identifies RSA and EC keys", () => {
  assert.equal(extractPublicKey(bytes(vectors.certificates.rsa))?.type, "RSA");
  for (const [name, curve] of [["p256", "P-256"], ["p384", "P-384"], ["p521", "P-521"]]) {
    const key = extractPublicKey(bytes(vectors.certificates[name]));
    assert.equal(key?.type, "EC");
    assert.equal(key?.curve, curve);
  }
});

test("extractPublicKey returns an SPKI that WebCrypto imports", async () => {
  const key = extractPublicKey(bytes(vectors.certificates.rsa));
  await subtle.importKey("spki", key.spki, { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" }, false, ["verify"]);
  assert.equal(key.spki[0], 0x30);
});

test("extractPublicKey rejects garbage, truncation and trailing bytes", () => {
  const der = bytes(vectors.certificates.p256);
  assert.equal(extractPublicKey(new Uint8Array()), null);
  assert.equal(extractPublicKey(new Uint8Array([1, 2, 3])), null);
  assert.equal(extractPublicKey(der.slice(0, der.length - 5)), null);
  assert.equal(extractPublicKey(Uint8Array.from([...der, 0])), null);
  assert.equal(extractPublicKey(der.slice(0, 40)), null);
});

for (const sig of vectors.signatures) {
  const hash = HASHES[sig.hash];
  const algorithm = sig.scheme.startsWith("rsa-pss") ? "RSASSA-PSS" : sig.scheme.startsWith("rsa") ? "RSASSA-PKCS1-v1_5" : "ECDSA";
  const certificateDer = bytes(vectors.certificates[sig.cert]);
  const raw = algorithm === "ECDSA" ? derToRaw(bytes(sig.der), SIZES[sig.scheme]) : bytes(sig.der);

  test(`verifies ${sig.scheme} ${hash}`, async () => {
    const args = { certificateDer, algorithm, hash, signature: raw, message };
    assert.equal(await verifySignature(subtle, args), "verified");
    const tampered = Uint8Array.from(raw);
    tampered[tampered.length - 1] ^= 1;
    assert.equal(await verifySignature(subtle, { ...args, signature: tampered }), "invalid");
    const other = new TextEncoder().encode("another message");
    assert.equal(await verifySignature(subtle, { ...args, message: other }), "invalid");
  });
}

test("a key that does not match the algorithm is unsupported, not a crash", async () => {
  const sig = vectors.signatures.find((s) => s.scheme === "p256" && s.hash === "sha256");
  const args = { certificateDer: bytes(vectors.certificates.p256), algorithm: "RSASSA-PSS", hash: "SHA-256", signature: bytes(sig.der), message };
  assert.equal(await verifySignature(subtle, args), "unsupported");
  assert.equal(await verifySignature(subtle, { ...args, algorithm: "ECDSA", hash: "MD5" }), "unsupported");
  assert.equal(await verifySignature(subtle, { ...args, certificateDer: new Uint8Array(3), algorithm: "ECDSA" }), "unsupported");
});

test("digestOf matches the known SHA-256 of the empty message", async () => {
  const digest = await digestOf(subtle, "SHA-256", new Uint8Array());
  assert.equal(Buffer.from(digest).toString("hex"), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855");
});
