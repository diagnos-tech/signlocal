import { describe, expect, it, vi } from "vitest";
import { loadSdk, settle, useFakeEnvironment } from "./helpers/env";
import { expectError } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { FINGERPRINT, wireCertificate } from "./helpers/fixtures";
import { beginSign } from "./helpers/sign";

useFakeEnvironment();

const prepare = () => new Uint8Array(32);

describe("sign() option validation (before anything is sent)", () => {
  const invalid: [string, unknown][] = [
    ["missing hash", { prepare }],
    ["lowercase hash", { hash: "sha-256", prepare }],
    ["SHA-1", { hash: "SHA-1", prepare }],
    ["MD5", { hash: "MD5", prepare }],
    ["numeric hash", { hash: 256, prepare }],
    ["array of hashes (no batch, D4)", { hash: ["SHA-256", "SHA-256"], prepare }],
    ["missing prepare", { hash: "SHA-256" }],
    ["prepare not a function", { hash: "SHA-256", prepare: "x" }],
    ["unknown algorithm", { hash: "SHA-256", algorithm: "EdDSA", prepare }],
    ["lowercase algorithm", { hash: "SHA-256", algorithm: "ecdsa", prepare }],
    ["empty algorithm list", { hash: "SHA-256", algorithm: [], prepare }],
    ["algorithm list with a bad entry", { hash: "SHA-256", algorithm: ["ECDSA", "RSA"], prepare }],
    ["fingerprint too short", { hash: "SHA-256", certificate: "abcd", prepare }],
    ["uppercase fingerprint", { hash: "SHA-256", certificate: FINGERPRINT.toUpperCase(), prepare }],
    ["certificate of the wrong kind", { hash: "SHA-256", certificate: 5, prepare }],
    ["certificate object without fingerprint", { hash: "SHA-256", certificate: {}, prepare }],
    ["no options at all", undefined],
    ["null options", null],
  ];
  for (const [name, options] of invalid) {
    it(`${name}: rejects InvalidRequest and posts nothing`, async () => {
      const env = await loadSdk();
      const s = settle(env.sdk.sign(options as never));
      await flush();
      expectError(env, s.outcome(), "InvalidRequest");
      expect(env.win.posted).toEqual([]);
    });
  }

  it("returns a rejected promise instead of throwing synchronously", async () => {
    const env = await loadSdk();
    let promise: Promise<unknown> | undefined;
    expect(() => {
      promise = env.sdk.sign({ hash: "nope", prepare } as never);
    }).not.toThrow();
    const s = settle(promise as Promise<unknown>);
    await flush();
    expectError(env, s.outcome(), "InvalidRequest");
  });

  it("does not call prepare for an invalid request", async () => {
    const env = await loadSdk();
    const fn = vi.fn(prepare);
    settle(env.sdk.sign({ hash: "SHA-1", prepare: fn } as never));
    await flush();
    expect(fn).not.toHaveBeenCalled();
  });
});

describe("sign.begin contents", () => {
  for (const hash of ["SHA-256", "SHA-384", "SHA-512"] as const) {
    it(`carries ${hash}`, async () => {
      const env = await loadSdk();
      await beginSign(env, { hash });
      expect(env.script.only("sign.begin").message.hash).toBe(hash);
    });
  }

  it("a single algorithm becomes a one-element list", async () => {
    const env = await loadSdk();
    await beginSign(env, { algorithm: "RSASSA-PSS" });
    expect(env.script.only("sign.begin").message.algorithms).toEqual(["RSASSA-PSS"]);
  });

  it("an algorithm list is deduplicated and keeps preference order", async () => {
    const env = await loadSdk();
    await beginSign(env, { algorithm: ["RSASSA-PSS", "ECDSA", "RSASSA-PSS"] });
    expect(env.script.only("sign.begin").message.algorithms).toEqual(["RSASSA-PSS", "ECDSA"]);
  });

  it("a fingerprint string is sent as is", async () => {
    const env = await loadSdk();
    await beginSign(env, { certificate: FINGERPRINT });
    expect(env.script.only("sign.begin").message.certificate).toBe(FINGERPRINT);
  });

  it("a Certificate object is reduced to its fingerprint: no certificate body leaves the page", async () => {
    const env = await loadSdk();
    env.script.autoReply = (request) =>
      request.message.type === "choose"
        ? [{ type: "choose.result", certificates: [wireCertificate()] }]
        : undefined;
    const c = settle(env.sdk.certificates());
    await flush();
    const cert = (c.outcome() as { value: unknown[] }).value[0];
    env.script.autoReply = () => undefined;
    await beginSign(env, { certificate: cert as never });
    const message = env.script.only("sign.begin").message;
    expect(message.certificate).toBe(FINGERPRINT);
    expect(JSON.stringify(message)).not.toContain("Ana Beatriz");
  });

  it("requests never carry origin or identity fields the page could forge", async () => {
    const env = await loadSdk();
    await beginSign(env);
    for (const request of env.script.requests) {
      for (const key of ["web", "origin", "topOrigin", "v", "caller"])
        expect(key in request.message).toBe(false);
    }
  });
});
