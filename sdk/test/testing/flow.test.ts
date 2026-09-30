import { createVerify, verify as nodeVerify, X509Certificate } from "node:crypto";
import { describe, expect, it } from "vitest";
import { load } from "./dom";

const data = new TextEncoder().encode("signed attributes of a test document");

describe("@websign/sdk/testing: the SDK signs through the fake", () => {
  const cases = [
    ["ECDSA", "SHA-256"],
    ["ECDSA", "SHA-384"],
    ["ECDSA", "SHA-512"],
    ["RSASSA-PKCS1-v1_5", "SHA-256"],
    ["RSASSA-PKCS1-v1_5", "SHA-512"],
    ["RSASSA-PSS", "SHA-256"],
    ["RSASSA-PSS", "SHA-384"],
  ] as const;
  for (const [algorithm, hash] of cases) {
    it(`${algorithm} with ${hash}: a signature Node's crypto verifies with the certificate`, async () => {
      const { sdk, testing } = await load();
      const fake = await testing.installFakeWebSign();
      const result = await sdk.sign({
        hash,
        algorithm,
        prepare: (_certificate, context) => crypto.subtle.digest(context.hash, data),
      });
      expect(result.algorithm).toBe(algorithm);
      expect(await fake.verify(result, data)).toBe(true);
      expect(await fake.verify(result, new TextEncoder().encode("other"))).toBe(false);

      // Independent check: the DER parses as X.509 and its key verifies the signature.
      const x509 = new X509Certificate(result.certificate.der);
      expect(x509.subject).toContain(result.certificate.displayName);
      const nodeHash = hash.replace("-", "").toLowerCase();
      const ok =
        algorithm === "ECDSA"
          ? nodeVerify(
              nodeHash,
              data,
              { key: x509.publicKey, dsaEncoding: "ieee-p1363" },
              result.signature,
            )
          : createVerify(nodeHash)
              .update(data)
              .verify(
                algorithm === "RSASSA-PSS"
                  ? { key: x509.publicKey, padding: 6, saltLength: result.digest.length }
                  : x509.publicKey,
                result.signature,
              );
      expect(ok).toBe(true);
      fake.uninstall();
    });
  }

  it("certificates are self-signed X.509 whose fingerprint is the SHA-256 of the DER", async () => {
    const { testing } = await load();
    const fake = await testing.installFakeWebSign();
    for (const certificate of fake.certificates) {
      const x509 = new X509Certificate(certificate.der);
      expect(x509.verify(x509.publicKey)).toBe(true);
      expect(x509.fingerprint256.replace(/:/g, "").toLowerCase()).toBe(certificate.fingerprint);
      expect(new Date(x509.validTo)).toEqual(certificate.notAfter);
    }
    expect(fake.certificates.map((c) => c.key.type)).toEqual(["EC", "RSA"]);
  });

  it("is deterministic: same certificates and signatures on every install", async () => {
    const run = async () => {
      const { sdk, testing } = await load();
      const fake = await testing.installFakeWebSign();
      const results = await Promise.all(
        (["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"] as const).map((algorithm) =>
          sdk.sign({
            hash: "SHA-256",
            algorithm,
            prepare: () => crypto.subtle.digest("SHA-256", data),
          }),
        ),
      );
      return {
        fingerprints: fake.certificates.map((c) => c.fingerprint),
        signatures: results.map((r) => Array.from(r.signature).join(",")),
      };
    };
    expect(await run()).toEqual(await run());
  });

  it("status() is ready and certificates() returns the chosen certificate", async () => {
    const { sdk, testing } = await load();
    const fake = await testing.installFakeWebSign({ remembered: true });
    expect(await sdk.status()).toMatchObject({ ready: true, remembered: true });
    expect(await sdk.certificates()).toEqual([fake.certificates[0]]);
    fake.choose(1);
    expect(await sdk.certificates()).toEqual([fake.certificates[1]]);
    expect(await sdk.certificates({ algorithm: "ECDSA" })).toEqual([fake.certificates[0]]);
  });

  it("records what the page asked, including the digest signed", async () => {
    const { sdk, testing } = await load();
    const fake = await testing.installFakeWebSign();
    const [ec] = fake.certificates;
    if (!ec) throw new Error("no default certificate");
    const result = await sdk.sign({
      hash: "SHA-256",
      certificate: ec,
      prepare: () => crypto.subtle.digest("SHA-256", data),
    });
    expect(fake.requests.at(-1)).toEqual({
      type: "sign",
      hash: "SHA-256",
      certificate: ec.fingerprint,
      digest: result.digest,
    });
  });

  it("the preselected certificate signs, and result.certificate equals fake.certificates[i]", async () => {
    const { sdk, testing } = await load();
    const fake = await testing.installFakeWebSign();
    const rsa = fake.certificates[1];
    if (!rsa) throw new Error("no default RSA certificate");
    const result = await sdk.sign({
      hash: "SHA-256",
      certificate: rsa.fingerprint,
      prepare: () => new Uint8Array(32),
    });
    expect(result.certificate).toEqual(rsa);
    expect(result.algorithm).toBe("RSASSA-PKCS1-v1_5");
  });
});
