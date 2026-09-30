import { describe, expect, it, vi } from "vitest";
import { loadSdk, settle, useFakeEnvironment } from "./helpers/env";
import { expectError, expectValue } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { wireError } from "./helpers/fixtures";

useFakeEnvironment();

describe("discovery", () => {
  it("posts one discover to its own origin and waits 1000 ms", async () => {
    const env = await loadSdk({ answerDiscover: false });
    const s = settle(env.sdk.status());
    await flush();
    expect(env.script.discovers).toEqual([{ source: "websign-page", kind: "discover" }]);
    expect(env.win.posted[0]?.targetOrigin).toBe(env.win.origin);
    await vi.advanceTimersByTimeAsync(998);
    expect(s.outcome()).toBeUndefined();
    await vi.advanceTimersByTimeAsync(2);
    expect(expectValue(s.outcome()).extension.installed).toBe(false);
  });

  it("does not post discover when an announcement was already seen", async () => {
    const env = await loadSdk({ answerDiscover: false });
    env.script.announce();
    env.script.autoReply = () => [
      {
        type: "status",
        extension: { version: "1.4.2", browser: "chrome" },
        appOutdated: false,
        remembered: false,
      },
    ];
    const s = settle(env.sdk.status());
    await flush();
    expect(env.script.discovers).toEqual([]);
    expect(s.outcome()?.ok).toBe(true);
  });

  it("resolves as soon as the announcement arrives, without waiting for the timeout", async () => {
    const env = await loadSdk({ answerDiscover: false });
    const s = settle(env.sdk.certificates());
    await flush();
    env.script.announce();
    await flush();
    expect(env.script.requestsOfType("choose")).toHaveLength(1);
    expect(s.outcome()).toBeUndefined();
  });

  it("shares one discovery among concurrent callers", async () => {
    const env = await loadSdk({ answerDiscover: false });
    settle(env.sdk.status());
    settle(env.sdk.status());
    settle(env.sdk.certificates());
    await flush();
    expect(env.script.discovers).toHaveLength(1);
  });

  it("caches a missing extension: a second call does not wait or post again", async () => {
    // The cached `null` is reused until a later announcement replaces it (SPEC §2).
    const env = await loadSdk({ answerDiscover: false });
    const first = settle(env.sdk.status());
    await vi.advanceTimersByTimeAsync(1000);
    expect(first.outcome()?.ok).toBe(true);
    const second = settle(env.sdk.status());
    await flush();
    expect(second.outcome()?.ok).toBe(true);
    expect(env.script.discovers).toHaveLength(1);
  });

  it("a late announcement replaces a cached miss", async () => {
    const env = await loadSdk({ answerDiscover: false });
    settle(env.sdk.status());
    await vi.advanceTimersByTimeAsync(1000);
    env.script.announce();
    env.script.autoReply = () => [
      {
        type: "status",
        extension: { version: "1.4.2", browser: "chrome" },
        appOutdated: false,
        remembered: false,
      },
    ];
    const s = settle(env.sdk.status());
    await flush();
    expect(expectValue(s.outcome()).extension).toEqual({ installed: true, version: "1.4.2" });
  });

  it("never sends a request before the extension announced itself", async () => {
    const env = await loadSdk({ answerDiscover: false });
    settle(env.sdk.certificates());
    settle(env.sdk.status());
    settle(env.sdk.sign({ hash: "SHA-256", prepare: () => new Uint8Array(32) }));
    await vi.advanceTimersByTimeAsync(900);
    expect(env.script.requests).toEqual([]);
    expect(env.script.discovers.length).toBeLessThanOrEqual(1);
  });

  it("certificates() and sign() reject ExtensionMissing after the timeout", async () => {
    const env = await loadSdk({ answerDiscover: false });
    const c = settle(env.sdk.certificates());
    const g = settle(env.sdk.sign({ hash: "SHA-256", prepare: () => new Uint8Array(32) }));
    await vi.advanceTimersByTimeAsync(1000);
    expectError(env, c.outcome(), "ExtensionMissing");
    expectError(env, g.outcome(), "ExtensionMissing");
    expect(env.script.requests).toEqual([]);
  });

  it("listens from import time and writes no globals", async () => {
    const env = await loadSdk({ answerDiscover: false });
    expect(env.importedGlobals).toEqual([]);
    expect(env.win.posted).toEqual([]);
  });
});

describe("status()", () => {
  const app = {
    version: "1.4.0",
    protocols: { min: 1, max: 1 },
    os: "linux",
    arch: "x86_64",
    channel: "direct",
  };
  const extension = { version: "1.4.2", browser: "chrome" };

  it("reports nothing installed without an extension", async () => {
    const env = await loadSdk({ answerDiscover: false });
    const s = settle(env.sdk.status());
    await vi.advanceTimersByTimeAsync(1000);
    expect(expectValue(s.outcome())).toEqual({
      extension: { installed: false },
      app: { installed: false, outdated: false },
      remembered: false,
      ready: false,
    });
  });

  it("maps a healthy status reply", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [
      { type: "status", extension, app, appOutdated: false, remembered: true },
    ];
    const s = settle(env.sdk.status());
    await flush();
    expect(env.script.only("status").message).toEqual({ type: "status" });
    expect(expectValue(s.outcome())).toEqual({
      extension: { installed: true, version: "1.4.2" },
      app: { installed: true, version: "1.4.0", outdated: false },
      remembered: true,
      ready: true,
    });
  });

  it("an outdated app is installed, outdated and not ready", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [
      { type: "status", extension, app, appOutdated: true, remembered: false },
    ];
    const s = settle(env.sdk.status());
    await flush();
    const status = expectValue(s.outcome());
    expect(status.app).toEqual({ installed: true, version: "1.4.0", outdated: true });
    expect(status.ready).toBe(false);
  });

  it("a status reply without app means the app is missing", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [
      { type: "status", extension, appOutdated: false, remembered: false },
    ];
    const s = settle(env.sdk.status());
    await flush();
    const status = expectValue(s.outcome());
    expect(status.app.installed).toBe(false);
    expect(status.extension.installed).toBe(true);
    expect(status.ready).toBe(false);
  });

  it("gives up after 5 s when the extension never answers status, and still resolves", async () => {
    const env = await loadSdk();
    const s = settle(env.sdk.status());
    await flush();
    const id = env.script.only("status").id;
    await vi.advanceTimersByTimeAsync(4999);
    expect(s.outcome()).toBeUndefined();
    await vi.advanceTimersByTimeAsync(1);
    const status = expectValue(s.outcome());
    expect(status.extension.installed).toBe(true);
    expect(status.app).toEqual({ installed: false, outdated: false });
    expect(status.ready).toBe(false);
    expect(env.script.only("cancel").id).toBe(id);
  });

  it("error AppMissing -> app not installed", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [wireError("AppMissing")];
    const s = settle(env.sdk.status());
    await flush();
    const status = expectValue(s.outcome());
    expect(status.app).toEqual({ installed: false, outdated: false });
    expect(status.extension.installed).toBe(true);
    expect(status.ready).toBe(false);
  });

  it("error AppOutdated -> installed and outdated", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [wireError("AppOutdated")];
    const s = settle(env.sdk.status());
    await flush();
    const status = expectValue(s.outcome());
    expect(status.app.installed).toBe(true);
    expect(status.app.outdated).toBe(true);
    expect(status.ready).toBe(false);
  });

  it("any other error -> installed, not outdated, and status still resolves", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [wireError("Internal")];
    const s = settle(env.sdk.status());
    await flush();
    const status = expectValue(s.outcome());
    expect(status.app.installed).toBe(true);
    expect(status.app.outdated).toBe(false);
  });

  it("is not ready when the extension speaks only other protocol versions", async () => {
    // An unsupported protocol range never yields ready (SPEC §4).
    const env = await loadSdk({ protocols: { min: 2, max: 3 } });
    env.script.autoReply = () => [
      { type: "status", extension, app, appOutdated: false, remembered: false },
    ];
    const s = settle(env.sdk.status());
    await flush();
    expect(expectValue(s.outcome()).ready).toBe(false);
  });
});
