import { describe, expect, it } from "vitest";
import { loadSdk, settle, useFakeEnvironment } from "./helpers/env";
import { expectValue } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { b64, bytes, needDigest, signResult, wireCertificate } from "./helpers/fixtures";
import { beginSign, deferred, digestsSent } from "./helpers/sign";

useFakeEnvironment();

describe("a newer need_digest supersedes a running prepare", () => {
  it("older prepare finishing last is discarded", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    const first = deferred<Uint8Array>();
    const second = deferred<Uint8Array>();
    prepare.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(id, needDigest(2));
    await flush();
    second.resolve(bytes(32, 2));
    await flush();
    first.resolve(bytes(32, 1));
    await flush();
    expect(digestsSent(env)).toEqual([{ seq: 2, digest: b64(bytes(32, 2)) }]);
  });

  it("older prepare finishing first is discarded too", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    const first = deferred<Uint8Array>();
    const second = deferred<Uint8Array>();
    prepare.mockReturnValueOnce(first.promise).mockReturnValueOnce(second.promise);
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(id, needDigest(2));
    await flush();
    first.resolve(bytes(32, 1));
    await flush();
    expect(digestsSent(env)).toEqual([]);
    second.resolve(bytes(32, 2));
    await flush();
    expect(digestsSent(env)).toEqual([{ seq: 2, digest: b64(bytes(32, 2)) }]);
  });

  it("a wrong-length or failing older prepare no longer ends the request", async () => {
    // Superseded results are discarded, including their failures (SPEC §6).
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env);
    const first = deferred<Uint8Array>();
    prepare.mockReturnValueOnce(first.promise).mockReturnValueOnce(bytes(32, 2));
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(id, needDigest(2));
    await flush();
    first.reject(new Error("stale"));
    await flush();
    expect(env.script.requestsOfType("cancel")).toEqual([]);
    expect(s.outcome()).toBeUndefined();
    env.script.reply(id, signResult());
    await flush();
    expectValue(s.outcome());
  });
});

describe("independent requests", () => {
  it("two sign() calls get distinct ids and replies are routed by id", async () => {
    const env = await loadSdk();
    const a = await beginSign(env);
    const secondPrepare = () => bytes(32, 0x33);
    const b = settle(env.sdk.sign({ hash: "SHA-256", prepare: secondPrepare }));
    await flush();
    const begins = env.script.requestsOfType("sign.begin");
    expect(begins).toHaveLength(2);
    const idB = begins[1]?.id as string;
    expect(idB).not.toBe(a.id);
    env.script.reply(idB, needDigest(1));
    await flush();
    expect(a.prepare).not.toHaveBeenCalled();
    expect(env.script.requestsOfType("sign.digest").map((r) => r.id)).toEqual([idB]);
    env.script.reply(idB, signResult({ signature: b64(Uint8Array.of(2)) }));
    await flush();
    expect(expectValue(b.outcome()).signature).toEqual(Uint8Array.of(2));
    expect(a.s.outcome()).toBeUndefined();
  });

  it("aborting one request leaves the other running", async () => {
    const env = await loadSdk();
    const ctl = new AbortController();
    const a = await beginSign(env, { signal: ctl.signal });
    const b = settle(env.sdk.sign({ hash: "SHA-256", prepare: () => bytes(32) }));
    await flush();
    ctl.abort();
    await flush();
    expect(env.script.only("cancel").id).toBe(a.id);
    expect(b.outcome()).toBeUndefined();
  });

  it("a Busy error for one request does not affect a running one", async () => {
    const env = await loadSdk();
    const a = await beginSign(env);
    const b = settle(env.sdk.sign({ hash: "SHA-256", prepare: () => bytes(32) }));
    await flush();
    const idB = env.script.requestsOfType("sign.begin")[1]?.id as string;
    env.script.reply(idB, { type: "error", code: "Busy", message: "queue full" });
    await flush();
    expect((b.outcome() as { error: { code: string } }).error.code).toBe("Busy");
    expect(a.s.outcome()).toBeUndefined();
  });

  it("a certificate in a reply is only ever handed to its own request's prepare", async () => {
    const env = await loadSdk();
    const a = await beginSign(env);
    const otherPrepare = () => bytes(32);
    settle(env.sdk.sign({ hash: "SHA-256", prepare: otherPrepare }));
    await flush();
    env.script.reply(
      a.id,
      needDigest(1, { certificate: wireCertificate({ fingerprint: "d".repeat(64) }) }),
    );
    await flush();
    expect(a.prepare).toHaveBeenCalledTimes(1);
  });
});

describe("no batch signing (D4)", () => {
  it("the public surface has no batch entry point", async () => {
    const env = await loadSdk();
    expect(Object.keys(env.sdk).filter((k) => /batch|many|multiple/i.test(k))).toEqual([]);
  });

  it("one sign() sends exactly one sign.begin", async () => {
    const env = await loadSdk();
    await beginSign(env);
    expect(env.script.requestsOfType("sign.begin")).toHaveLength(1);
  });

  it("a result never triggers another digest request by itself", async () => {
    const env = await loadSdk();
    const { id } = await beginSign(env);
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(id, signResult());
    await flush();
    expect(digestsSent(env)).toHaveLength(1);
  });
});
