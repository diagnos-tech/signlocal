import { afterEach, describe, expect, it } from "vitest";
import { WebSignError } from "../src/index";
import {
  type FakeApp,
  type FakeCertificate,
  fakeApp,
  SAMPLE_CERTIFICATE,
} from "../src/testing/index";

const apps: FakeApp[] = [];
const start = (...args: Parameters<typeof fakeApp>) => {
  const app = fakeApp(...args);
  apps.push(app);
  return app;
};
afterEach(async () => {
  await Promise.all(apps.splice(0).map((app) => app.close()));
});

const digest = (length: number) => new Uint8Array(length).fill(9);

describe("fakeApp", () => {
  it("lets the public API run end to end with no installed app", async () => {
    const app = start();
    const websign = await app.connect();
    const result = await websign.sign({ hash: "SHA-256", prepare: () => digest(32) });
    expect(result.certificate.displayName).toBe(SAMPLE_CERTIFICATE.displayName);
    expect(result.algorithm).toBe("ECDSA");
    // The fake echoes the digest: it is a stand-in, never a valid signature.
    expect([...result.signature]).toEqual([...digest(32)]);
    expect(app.requests().map((r) => r.type)).toEqual(["hello", "sign.begin", "sign.digest"]);
  });

  it("answers status, certificates and diagnostics", async () => {
    const app = start({ remembered: true });
    const websign = await app.connect();
    expect((await websign.status()).remembered).toBe(true);
    const [certificate] = await websign.certificates();
    expect(certificate?.fingerprint).toBe(SAMPLE_CERTIFICATE.fingerprint);
    await expect(websign.openDiagnostics("help")).resolves.toBeUndefined();
  });

  it("offers your certificates and honours the algorithm filter", async () => {
    const rsa: FakeCertificate = {
      ...SAMPLE_CERTIFICATE,
      fingerprint: "cd".repeat(32),
      key: { type: "RSA", bits: 2048 },
      algorithms: ["RSASSA-PKCS1-v1_5"],
    };
    const app = start({ certificates: [SAMPLE_CERTIFICATE, rsa] });
    const websign = await app.connect();
    const chosen = await websign.certificates({ algorithm: "RSASSA-PKCS1-v1_5" });
    expect(chosen.map((c) => c.fingerprint)).toEqual([rsa.fingerprint]);
    const signed = await websign.sign({
      hash: "SHA-256",
      certificate: rsa.fingerprint,
      prepare: () => digest(32),
    });
    expect(signed.algorithm).toBe("RSASSA-PKCS1-v1_5");
  });

  it("reports the failure you ask for, with code, hint and docs link", async () => {
    const app = start({ failWith: { code: "UserCancelled" } });
    const websign = await app.connect();
    const error = await websign
      .sign({ hash: "SHA-256", prepare: () => digest(32) })
      .catch((e) => e);
    expect(error).toBeInstanceOf(WebSignError);
    expect(error).toMatchObject({ code: "UserCancelled" });
    expect(error.hint).toMatch(/Cancel/);
    expect(error.docsUrl).toMatch(/^https:\/\//);
  });

  it("can fail after prepare, like a blocked PIN", async () => {
    const app = start({ failWith: { code: "PinLocked", when: "confirm" } });
    const websign = await app.connect();
    let prepared = 0;
    const error = await websign
      .sign({
        hash: "SHA-256",
        prepare: () => {
          prepared += 1;
          return digest(32);
        },
      })
      .catch((e) => e);
    expect(prepared).toBe(1);
    expect(error).toMatchObject({ code: "PinLocked" });
  });

  it("enforces the digest length like the app", async () => {
    const app = start();
    const websign = await app.connect();
    const error = await websign
      .sign({ hash: "SHA-384", prepare: () => digest(32) })
      .catch((e) => e);
    expect(error).toMatchObject({ code: "InvalidRequest" });
  });

  it("reports NoCertificates when nothing matches", async () => {
    const app = start();
    const websign = await app.connect();
    const error = await websign.certificates({ algorithm: "RSASSA-PSS" }).catch((e) => e);
    expect(error).toMatchObject({ code: "NoCertificates" });
  });

  it("removes its scratch files on close and closes its connections", async () => {
    const app = start();
    const websign = await app.connect();
    await app.close();
    await expect(websign.status()).rejects.toBeInstanceOf(WebSignError);
  });
});
