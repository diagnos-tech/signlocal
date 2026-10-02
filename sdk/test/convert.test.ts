import { afterEach, describe, expect, it, vi } from "vitest";
import { fromBase64, toBase64, toCertificate } from "../src/convert";
import { WebSignError } from "../src/errors";
import { b64, wireCertificate } from "./helpers/fixtures";

const text = (s: string) => Uint8Array.from(s, (c) => c.charCodeAt(0));

afterEach(() => vi.unstubAllGlobals());

describe("Base64 (RFC 4648 section 4, padded, strict)", () => {
  const vectors: [string, string][] = [
    ["", ""],
    ["f", "Zg=="],
    ["fo", "Zm8="],
    ["foo", "Zm9v"],
    ["foob", "Zm9vYg=="],
    ["fooba", "Zm9vYmE="],
    ["foobar", "Zm9vYmFy"],
  ];
  for (const [plain, encoded] of vectors) {
    it(`encodes and decodes ${JSON.stringify(plain)}`, () => {
      expect(toBase64(text(plain))).toBe(encoded);
      expect(fromBase64(encoded)).toEqual(text(plain));
    });
  }

  it("uses the standard alphabet (+ and /)", () => {
    expect(toBase64(Uint8Array.of(0xfb, 0xff, 0xbf))).toBe("+/+/");
    expect(fromBase64("+/+/")).toEqual(Uint8Array.of(0xfb, 0xff, 0xbf));
  });

  it("round-trips every length up to 70 over the full byte range", () => {
    for (let n = 0; n <= 70; n++) {
      const data = Uint8Array.from({ length: n }, (_, i) => (i * 37 + n) & 0xff);
      expect(fromBase64(toBase64(data))).toEqual(data);
    }
  });

  it("handles large inputs", () => {
    const data = Uint8Array.from({ length: 600_000 }, (_, i) => i & 0xff);
    const decoded = fromBase64(toBase64(data));
    // Element-wise: toEqual on 600k elements alone takes seconds.
    expect(decoded.length).toBe(data.length);
    expect(decoded.every((byte, i) => byte === data[i])).toBe(true);
  });

  it("encodes only the bytes of a view", () => {
    const backing = Uint8Array.of(9, 9, 102, 111, 111, 9);
    expect(toBase64(backing.subarray(2, 5))).toBe("Zm9v");
  });

  const invalid: [string, string][] = [
    ["missing padding", "Zg"],
    ["missing one padding char", "Zm8"],
    ["length not a multiple of 4", "Zm9vY"],
    ["too much padding", "Zg==="],
    ["padding in the middle", "Zg==Zg=="],
    ["only padding", "===="],
    ["three padding chars in a quad", "Z==="],
    ["whitespace", "Zm9v Zg=="],
    ["trailing newline", "Zm9v\n"],
    ["leading space", " Zm9v"],
    ["URL-safe alphabet", "-_-_"],
    ["invalid character", "Zm9*"],
    ["non-zero trailing bits (one byte)", "Zh=="],
    ["non-zero trailing bits (two bytes)", "Zm9="],
    ["non-ASCII", "Zm9é"],
  ];
  for (const [name, input] of invalid) {
    it(`rejects ${name} as Internal (only replies are decoded)`, () => {
      expect(() => fromBase64(input)).toThrow(WebSignError);
      expect(() => fromBase64(input)).toThrow(expect.objectContaining({ code: "Internal" }));
    });
  }

  it("round-trips every byte value", () => {
    const all = Uint8Array.from({ length: 256 }, (_, i) => i);
    expect(fromBase64(toBase64(all))).toEqual(all);
  });
});

describe("toCertificate", () => {
  it("decodes bytes, converts Unix seconds to Date and copies the rest", () => {
    const cert = toCertificate(
      wireCertificate({ key: { type: "EC", curve: "P-256" }, algorithms: ["ECDSA"] }),
    );
    expect(cert.der).toEqual(Uint8Array.of(1, 2, 3, 4));
    expect(cert.chain).toEqual([Uint8Array.of(9, 8, 7)]);
    expect(cert.notBefore.getTime()).toBe(1741000000_000);
    expect(cert.notAfter.getTime()).toBe(1792000000_000);
    expect(cert.fingerprint).toHaveLength(64);
    expect(cert.displayName).toBe("Ana Beatriz Souza");
    expect(cert.issuerName).toBe("AC SOLUTI Multipla v5");
    expect(cert.key).toEqual({ type: "EC", curve: "P-256" });
    expect(cert.algorithms).toEqual(["ECDSA"]);
    expect(cert.profile).toEqual({ icpBrasil: "A3", keyStorage: "hardware" });
  });

  it("keeps an empty chain empty", () => {
    expect(toCertificate(wireCertificate({ chain: [] })).chain).toEqual([]);
  });

  it("copies eidas.types without sharing the wire array", () => {
    const wire = wireCertificate({
      profile: {
        keyStorage: "unknown",
        eidas: { qualified: true, qscd: true, types: ["esign", "eseal"] },
      },
    });
    const cert = toCertificate(wire);
    expect(cert.profile.eidas).toEqual({ qualified: true, qscd: true, types: ["esign", "eseal"] });
    expect(cert.profile.eidas?.types).not.toBe(wire.profile.eidas?.types);
  });

  it("leaves profile fields absent when the wire omits them", () => {
    const cert = toCertificate(wireCertificate({ profile: { keyStorage: "software" } }));
    expect("icpBrasil" in cert.profile).toBe(false);
    expect("eidas" in cert.profile).toBe(false);
  });

  it("rejects a certificate with invalid Base64", () => {
    expect(() => toCertificate(wireCertificate({ der: "@@" }))).toThrow();
    expect(() =>
      toCertificate(wireCertificate({ chain: [b64(Uint8Array.of(1)), "@@"] })),
    ).toThrow();
  });
});
