import { describe, expect, it } from "vitest";
import { loadSdk, settle, useFakeEnvironment } from "./helpers/env";
import { expectError, expectValue } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { needDigest, pageStatus, signResult, wireCertificate } from "./helpers/fixtures";

useFakeEnvironment();

// The SDK speaks page-message protocol version 1. An extension whose range starts above 1
// needs a newer SDK: requests are refused locally with ClientOutdated (the site must update
// @websign/sdk) and nothing but `discover` is ever posted (SPEC §2, T9).
const UNKNOWN: [string, { min: number; max: number }][] = [
  ["only newer versions", { min: 2, max: 2 }],
  ["a far newer range", { min: 5, max: 9 }],
  ["a range that starts above 1", { min: 2, max: 3 }],
];

describe("protocol version negotiation", () => {
  for (const [name, protocols] of UNKNOWN) {
    it(`certificates() refuses ${name}`, async () => {
      const env = await loadSdk({ protocols });
      const s = settle(env.sdk.certificates());
      await flush();
      expectError(env, s.outcome(), "ClientOutdated");
      expect(env.script.requests).toEqual([]);
    });

    it(`sign() refuses ${name} without calling prepare`, async () => {
      const env = await loadSdk({ protocols });
      let called = false;
      const s = settle(
        env.sdk.sign({
          hash: "SHA-256",
          prepare: () => {
            called = true;
            return new Uint8Array(32);
          },
        }),
      );
      await flush();
      expectError(env, s.outcome(), "ClientOutdated");
      expect(called).toBe(false);
      expect(env.script.requests).toEqual([]);
    });

    it(`status() reports the extension but never ready for ${name}`, async () => {
      const env = await loadSdk({ protocols });
      env.script.autoReply = () => [pageStatus()];
      const s = settle(env.sdk.status());
      await flush();
      const status = expectValue(s.outcome());
      expect(status.extension.installed).toBe(true);
      expect(status.ready).toBe(false);
    });
  }

  for (const [name, protocols] of [
    ["exactly 1", { min: 1, max: 1 }],
    ["a range that includes 1", { min: 1, max: 4 }],
  ] as const) {
    it(`talks to an extension speaking ${name}`, async () => {
      const env = await loadSdk({ protocols });
      const s = settle(env.sdk.certificates());
      await flush();
      expect(env.script.requestsOfType("choose")).toHaveLength(1);
      env.script.reply(env.script.only("choose").id, {
        type: "choose.result",
        certificates: [wireCertificate()],
      });
      await flush();
      expect(s.outcome()?.ok).toBe(true);
    });
  }

  it("requests carry no version field: page messages are versioned by the announcement only", async () => {
    const env = await loadSdk();
    settle(env.sdk.certificates());
    await flush();
    const request = env.script.only("choose");
    expect(Object.keys(request).sort()).toEqual(["id", "kind", "message", "source"]);
  });

  it("ClientOutdated names the SDK's version and the one required", async () => {
    const env = await loadSdk({ protocols: { min: 2, max: 3 } });
    const s = settle(env.sdk.certificates());
    await flush();
    expect(expectError(env, s.outcome(), "ClientOutdated").details).toEqual({
      installed: "1",
      required: "2",
    });
  });

  it("an upgrade announcement that drops version 1 stops later requests", async () => {
    const env = await loadSdk();
    env.script.announce();
    env.win.deliver({ data: { ...env.script.announcement(), protocols: { min: 2, max: 2 } } });
    const s = settle(env.sdk.certificates());
    await flush();
    expectError(env, s.outcome(), "ClientOutdated");
    expect(env.script.requests).toEqual([]);
  });

  it("replies carrying unknown extra fields do not break a supported flow", async () => {
    // Strictness is enforced by the extension/app; the SDK reads the fields it knows.
    const env = await loadSdk();
    const s = settle(env.sdk.sign({ hash: "SHA-256", prepare: () => new Uint8Array(32) }));
    await flush();
    const id = env.script.only("sign.begin").id;
    env.script.reply(id, needDigest(1, { future: true }));
    await flush();
    env.script.reply(id, signResult({ future: true }));
    await flush();
    expect(s.outcome()?.ok).toBe(true);
  });
});
