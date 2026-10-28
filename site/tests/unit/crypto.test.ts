import { describe, expect, test } from "bun:test";
import { digestOf, extractPublicKey, verifySignature } from "../../src/client/test-page/crypto.ts";
import { SAMPLE_TEXT } from "../../src/client/test-page/state.ts";
import vectors from "../fixtures/vectors.json";

const subtle = globalThis.crypto.subtle;
const message = new TextEncoder().encode(vectors.message);
const bytes = (base64: string) => new Uint8Array(Buffer.from(base64, "base64"));
const certificates = vectors.certificates as Record<string, string>;

/** openssl writes ECDSA as DER SEQUENCE { r, s }; WebCrypto and the SDK use r||s. */
function derToRaw(der: Uint8Array, size: number): Uint8Array {
  const read = (pos: number) => {
    let length = der[pos + 1] ?? 0;
    let start = pos + 2;
    if (length & 0x80) {
      const count = length & 0x7f;
      length = 0;
      for (let i = 0; i < count; i++) length = length * 256 + (der[start + i] ?? 0);
      start += count;
    }
    return { start, end: start + length };
  };
  const sequence = read(0);
  const r = read(sequence.start);
  const s = read(r.end);
  const pad = (value: { start: number; end: number }) => {
    const raw = der.slice(value.start, value.end);
    const trimmed = raw.slice(raw.findIndex((byte) => byte !== 0));
    const out = new Uint8Array(size);
    out.set(trimmed, size - trimmed.length);
    return out;
  };
  return Uint8Array.from([...pad(r), ...pad(s)]);
}

const HASHES = { sha256: "SHA-256", sha384: "SHA-384", sha512: "SHA-512" } as const;
const SIZES = { p256: 32, p384: 48, p521: 66 } as const;

describe("the vectors", () => {
  test("sign the page's sample text", () => {
    expect(vectors.message).toBe(SAMPLE_TEXT);
  });
});

describe("extractPublicKey", () => {
  test("identifies RSA and EC keys", () => {
    expect(extractPublicKey(bytes(certificates.rsa ?? ""))?.type).toBe("RSA");
    for (const [name, curve] of [
      ["p256", "P-256"],
      ["p384", "P-384"],
      ["p521", "P-521"],
    ] as const) {
      const key = extractPublicKey(bytes(certificates[name] ?? ""));
      expect(key?.type).toBe("EC");
      expect(key && "curve" in key ? key.curve : undefined).toBe(curve);
    }
  });

  test("returns an SPKI that WebCrypto imports", async () => {
    const key = extractPublicKey(bytes(certificates.rsa ?? ""));
    expect(key).not.toBeNull();
    await subtle.importKey(
      "spki",
      key?.spki as BufferSource,
      { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" },
      false,
      ["verify"],
    );
    expect(key?.spki[0]).toBe(0x30);
  });

  test("rejects garbage, truncation and trailing bytes", () => {
    const der = bytes(certificates.p256 ?? "");
    expect(extractPublicKey(new Uint8Array())).toBeNull();
    expect(extractPublicKey(new Uint8Array([1, 2, 3]))).toBeNull();
    expect(extractPublicKey(der.slice(0, der.length - 5))).toBeNull();
    expect(extractPublicKey(Uint8Array.from([...der, 0]))).toBeNull();
    expect(extractPublicKey(der.slice(0, 40))).toBeNull();
  });
});

describe("verifySignature", () => {
  for (const sig of vectors.signatures) {
    const hash = HASHES[sig.hash as keyof typeof HASHES];
    const algorithm = sig.scheme.startsWith("rsa-pss")
      ? "RSASSA-PSS"
      : sig.scheme.startsWith("rsa")
        ? "RSASSA-PKCS1-v1_5"
        : "ECDSA";
    const certificateDer = bytes(certificates[sig.cert] ?? "");
    const raw =
      algorithm === "ECDSA"
        ? derToRaw(bytes(sig.der), SIZES[sig.scheme as keyof typeof SIZES])
        : bytes(sig.der);

    test(`verifies ${sig.scheme} ${hash}, and refuses a changed signature or message`, async () => {
      const input = { certificateDer, algorithm, hash, signature: raw, message };
      expect(await verifySignature(subtle, input)).toBe("verified");
      const tampered = Uint8Array.from(raw);
      tampered[tampered.length - 1] = (tampered[tampered.length - 1] ?? 0) ^ 1;
      expect(await verifySignature(subtle, { ...input, signature: tampered })).toBe("invalid");
      const other = new TextEncoder().encode("another message");
      expect(await verifySignature(subtle, { ...input, message: other })).toBe("invalid");
    });
  }

  test("a key that does not match the algorithm is unsupported, not a crash", async () => {
    const sig = vectors.signatures.find(
      (entry) => entry.scheme === "p256" && entry.hash === "sha256",
    );
    const input = {
      certificateDer: bytes(certificates.p256 ?? ""),
      algorithm: "RSASSA-PSS",
      hash: "SHA-256",
      signature: bytes(sig?.der ?? ""),
      message,
    };
    expect(await verifySignature(subtle, input)).toBe("unsupported");
    expect(await verifySignature(subtle, { ...input, algorithm: "ECDSA", hash: "MD5" })).toBe(
      "unsupported",
    );
    expect(
      await verifySignature(subtle, {
        ...input,
        certificateDer: new Uint8Array(3),
        algorithm: "ECDSA",
      }),
    ).toBe("unsupported");
  });
});

describe("digestOf", () => {
  test("matches the known SHA-256 of the empty message", async () => {
    const digest = await digestOf(subtle, "SHA-256", new Uint8Array());
    expect(Buffer.from(digest).toString("hex")).toBe(
      "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
    );
  });
});
