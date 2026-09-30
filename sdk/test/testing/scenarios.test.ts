import { describe, expect, it, vi } from "vitest";
import { load } from "./dom";

const prepare = () => new Uint8Array(32);
const signOnce = (sdk: Awaited<ReturnType<typeof load>>["sdk"]) =>
  sdk.sign({ hash: "SHA-256", prepare });

describe("@websign/sdk/testing scenarios", () => {
  const broken = [
    ["app-missing", "AppMissing", { app: { installed: false, outdated: false } }],
    ["app-outdated", "AppOutdated", { app: { installed: true, version: "0.9.0", outdated: true } }],
    ["extension-outdated", "ExtensionOutdated", { ready: true }],
    ["sdk-outdated", "ClientOutdated", { problem: "ClientOutdated" }],
  ] as const;
  for (const [scenario, code, status] of broken) {
    it(`${scenario}: status() says so and sign() rejects ${code}`, async () => {
      const { sdk, testing } = await load();
      await testing.installFakeWebSign({ scenario });
      expect(await sdk.status()).toMatchObject(status);
      await expect(signOnce(sdk)).rejects.toMatchObject({ name: "WebSignError", code });
    });
  }

  it("app-outdated carries the versions, which errorText fills in", async () => {
    const { sdk, testing } = await load();
    await testing.installFakeWebSign({ scenario: "app-outdated" });
    const error = await signOnce(sdk).catch((e: unknown) => e);
    expect(error).toMatchObject({ details: { installed: "0.9.0", required: "1.0.0" } });
  });

  it("extension-missing: status() waits for discovery, then sign() rejects ExtensionMissing", async () => {
    const { sdk, testing } = await load();
    const fake = await testing.installFakeWebSign({ scenario: "extension-missing" });
    expect(await sdk.status()).toMatchObject({ ready: false, problem: "ExtensionMissing" });
    await expect(signOnce(sdk)).rejects.toMatchObject({ code: "ExtensionMissing" });
    expect(fake.requests).toEqual([]);
    expect(() => fake.setScenario("extension-missing" as "ready")).toThrow(
      /never loses its extension/,
    );
  });

  it("failNext: the next certificates()/sign() fails once, status is untouched", async () => {
    const { sdk, testing } = await load();
    const fake = await testing.installFakeWebSign();
    fake.failNext("UserCancelled");
    expect((await sdk.status()).ready).toBe(true);
    await expect(signOnce(sdk)).rejects.toMatchObject({ code: "UserCancelled" });
    await expect(signOnce(sdk)).resolves.toMatchObject({ algorithm: "ECDSA" });
    fake.failNext("DriverFailure", { native: "CKR_DEVICE_ERROR (0x00000030)" });
    await expect(sdk.certificates()).rejects.toMatchObject({
      code: "DriverFailure",
      details: { native: "CKR_DEVICE_ERROR (0x00000030)" },
    });
  });

  it("no certificates: certificates() and sign() reject NoCertificates", async () => {
    const { sdk, testing } = await load();
    await testing.installFakeWebSign({ certificates: [] });
    await expect(sdk.certificates()).rejects.toMatchObject({ code: "NoCertificates" });
    await expect(signOnce(sdk)).rejects.toMatchObject({ code: "NoCertificates" });
  });

  it("an algorithm no certificate supports rejects NoCertificates", async () => {
    const { sdk, testing } = await load();
    await testing.installFakeWebSign({ certificates: [{ key: "EC" }] });
    await expect(
      sdk.sign({ hash: "SHA-256", algorithm: "RSASSA-PSS", prepare }),
    ).rejects.toMatchObject({ code: "NoCertificates" });
  });

  it("an expired certificate rejects CertificateNotValid", async () => {
    const { sdk, testing } = await load();
    await testing.installFakeWebSign({ certificates: [{ notAfter: new Date("2020-01-01") }] });
    await expect(signOnce(sdk)).rejects.toMatchObject({ code: "CertificateNotValid" });
  });

  it("a preselected fingerprint the person does not have rejects CertificateUnavailable", async () => {
    const { sdk, testing } = await load();
    await testing.installFakeWebSign();
    const certificate = "0".repeat(64);
    await expect(sdk.sign({ hash: "SHA-256", certificate, prepare })).rejects.toMatchObject({
      code: "CertificateUnavailable",
    });
  });

  it("switchDuringNextSign: prepare runs for both certificates and the second one signs", async () => {
    const { sdk, testing } = await load();
    const fake = await testing.installFakeWebSign();
    fake.switchDuringNextSign(1);
    const seen: string[] = [];
    const result = await sdk.sign({
      hash: "SHA-256",
      prepare: (certificate) => {
        seen.push(certificate.fingerprint);
        return new Uint8Array(32).fill(seen.length);
      },
    });
    expect(seen).toEqual(fake.certificates.map((c) => c.fingerprint));
    expect(result.certificate).toEqual(fake.certificates[1]);
    expect(result.digest).toEqual(new Uint8Array(32).fill(2));
    await expect(signOnce(sdk)).resolves.toMatchObject({ certificate: fake.certificates[0] });
  });

  it("setScenario re-announces, so onChange listeners see the new status", async () => {
    const { sdk, testing } = await load();
    const fake = await testing.installFakeWebSign();
    const seen = vi.fn();
    const stop = sdk.onChange(seen);
    fake.setScenario("app-missing");
    await vi.waitFor(() =>
      expect(seen).toHaveBeenCalledWith(expect.objectContaining({ problem: "AppMissing" })),
    );
    stop();
  });

  it("an abort cancels the fake's request", async () => {
    const { sdk, testing } = await load();
    const fake = await testing.installFakeWebSign();
    const controller = new AbortController();
    const signing = sdk.sign({
      hash: "SHA-256",
      signal: controller.signal,
      prepare: () => {
        controller.abort();
        return new Uint8Array(32);
      },
    });
    await expect(signing).rejects.toMatchObject({ code: "Aborted" });
    await vi.waitFor(() => expect(fake.requests.at(-1)).toMatchObject({ cancelled: true }));
    expect(fake.requests.at(-1)?.digest).toBeUndefined();
  });

  it("latencyMs delays every answer", async () => {
    const { sdk, testing } = await load();
    await testing.installFakeWebSign({ latencyMs: 60 });
    const started = performance.now();
    await sdk.certificates();
    expect(performance.now() - started).toBeGreaterThanOrEqual(55);
  });

  it("uninstall stops answering; a new install replaces the old fake", async () => {
    const { sdk, testing } = await load();
    const first = await testing.installFakeWebSign({ certificates: [{ displayName: "First" }] });
    const second = await testing.installFakeWebSign({ certificates: [{ displayName: "Second" }] });
    const [chosen] = await sdk.certificates();
    expect(chosen?.displayName).toBe("Second");
    expect(first.requests).toEqual([]);
    second.uninstall();
    const controller = new AbortController();
    const pending = sdk.certificates({ signal: controller.signal });
    setTimeout(() => controller.abort(), 50);
    await expect(pending).rejects.toMatchObject({ code: "Aborted" });
  });
});
