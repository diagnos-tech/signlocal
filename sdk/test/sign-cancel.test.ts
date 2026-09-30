import { describe, expect, it, vi } from "vitest";
import { loadSdk, settle, useFakeEnvironment } from "./helpers/env";
import { expectError } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { bytes, needDigest, signResult, wireError } from "./helpers/fixtures";
import { beginSign, deferred, digestsSent } from "./helpers/sign";

useFakeEnvironment();

describe("sign() cancellation", () => {
  it("abort while waiting posts cancel with the same id and rejects Aborted immediately", async () => {
    const env = await loadSdk();
    const ctl = new AbortController();
    const { s, id } = await beginSign(env, { signal: ctl.signal });
    ctl.abort();
    await flush();
    expect(env.script.only("cancel")).toMatchObject({ id, message: { type: "cancel" } });
    expectError(env, s.outcome(), "Aborted");
  });

  it("does not wait for the app's error reply after aborting", async () => {
    const env = await loadSdk();
    const ctl = new AbortController();
    const { s } = await beginSign(env, { signal: ctl.signal });
    ctl.abort();
    await flush();
    expect(s.outcome()).toBeDefined();
  });

  it("abort while prepare runs: cancel, Aborted, and the late digest is never sent", async () => {
    const env = await loadSdk();
    const ctl = new AbortController();
    const slow = deferred<Uint8Array>();
    const { s, prepare, id } = await beginSign(env, { signal: ctl.signal });
    prepare.mockReturnValue(slow.promise);
    env.script.reply(id, needDigest(1));
    await flush();
    ctl.abort();
    await flush();
    slow.resolve(bytes(32));
    await flush();
    expect(env.script.requestsOfType("cancel")).toHaveLength(1);
    expect(digestsSent(env)).toEqual([]);
    expectError(env, s.outcome(), "Aborted");
  });

  it("an already aborted signal rejects Aborted, calls nothing and posts nothing", async () => {
    const env = await loadSdk();
    const prepare = vi.fn(() => bytes(32));
    const s = settle(env.sdk.sign({ hash: "SHA-256", prepare, signal: AbortSignal.abort() }));
    await flush();
    expectError(env, s.outcome(), "Aborted");
    expect(prepare).not.toHaveBeenCalled();
    expect(env.win.posted).toEqual([]);
  });

  it("abort while the extension is still being discovered posts no request", async () => {
    const env = await loadSdk({ answerDiscover: false });
    const ctl = new AbortController();
    const s = settle(
      env.sdk.sign({ hash: "SHA-256", prepare: () => bytes(32), signal: ctl.signal }),
    );
    await flush();
    ctl.abort();
    await vi.advanceTimersByTimeAsync(1000);
    expectError(env, s.outcome(), "Aborted");
    expect(env.script.requests).toEqual([]);
  });

  it("abort after the request settled does nothing", async () => {
    const env = await loadSdk();
    const ctl = new AbortController();
    const { s, id } = await beginSign(env, { signal: ctl.signal });
    env.script.reply(id, signResult());
    await flush();
    ctl.abort();
    await flush();
    expect(s.outcome()?.ok).toBe(true);
    expect(env.script.requestsOfType("cancel")).toEqual([]);
  });

  it("abort after an error does not post cancel", async () => {
    const env = await loadSdk();
    const ctl = new AbortController();
    const { s, id } = await beginSign(env, { signal: ctl.signal });
    env.script.reply(id, wireError("UserCancelled"));
    await flush();
    ctl.abort();
    await flush();
    expectError(env, s.outcome(), "UserCancelled");
    expect(env.script.requestsOfType("cancel")).toEqual([]);
  });

  it("aborting twice posts a single cancel", async () => {
    const env = await loadSdk();
    const ctl = new AbortController();
    await beginSign(env, { signal: ctl.signal });
    ctl.abort();
    ctl.abort();
    await flush();
    expect(env.script.requestsOfType("cancel")).toHaveLength(1);
  });

  it("removes its abort listener once settled", async () => {
    const env = await loadSdk();
    const ctl = new AbortController();
    const add = vi.spyOn(ctl.signal, "addEventListener");
    const remove = vi.spyOn(ctl.signal, "removeEventListener");
    const { id } = await beginSign(env, { signal: ctl.signal });
    env.script.reply(id, signResult());
    await flush();
    expect(add.mock.calls.length).toBeGreaterThan(0);
    expect(remove.mock.calls.length).toBeGreaterThanOrEqual(add.mock.calls.length);
  });
});
