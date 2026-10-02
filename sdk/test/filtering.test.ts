import { describe, expect, it, vi } from "vitest";
import { loadSdk, settle, useFakeEnvironment } from "./helpers/env";
import { expectValue } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { ForeignWindow } from "./helpers/fake-window";
import { needDigest, pageStatus, wireCertificate } from "./helpers/fixtures";
import { beginSign, digestsSent } from "./helpers/sign";

useFakeEnvironment();

const announce = (extra: Record<string, unknown> = {}) => ({
  source: "websign-extension",
  kind: "announce",
  extension: { version: "9.9.9", browser: "chrome" },
  protocols: { min: 1, max: 1 },
  ...extra,
});

const frame = (id: string, message: unknown, extra: Record<string, unknown> = {}) => ({
  source: "websign-extension",
  kind: "message",
  id,
  message,
  ...extra,
});

describe("announcements from anyone but the content script are ignored", () => {
  const forged: [string, { data: unknown; source?: unknown; origin?: string }][] = [
    ["another window (iframe)", { data: announce(), source: new ForeignWindow() }],
    ["a null source", { data: announce(), source: null }],
    ["another origin", { data: announce(), origin: "https://evil.example" }],
    ["a sibling subdomain", { data: announce(), origin: "https://sub.app.example" }],
    ["the opaque origin", { data: announce(), origin: "null" }],
    ["the page's own source tag", { data: announce({ source: "websign-page" }) }],
    ["no source tag", { data: announce({ source: undefined }) }],
    ["an unknown source tag", { data: announce({ source: "websign-extension2" }) }],
  ];
  for (const [name, event] of forged) {
    it(`${name}`, async () => {
      const env = await loadSdk({ answerDiscover: false });
      env.win.deliver(event);
      const s = settle(env.sdk.status());
      await advance();
      expect(expectValue(s.outcome()).extension.installed).toBe(false);
    });
  }

  const malformed: [string, unknown][] = [
    ["null", null],
    ["a string", "websign-extension"],
    ["a number", 42],
    ["an array", [announce()]],
    ["an empty object", {}],
    ["an unknown kind", announce({ kind: "hello" })],
    ["a missing extension", announce({ extension: undefined })],
    ["a non-object extension", announce({ extension: "1.0" })],
    ["a non-string version", announce({ extension: { version: 1, browser: "chrome" } })],
    ["missing protocols", announce({ protocols: undefined })],
    ["min above max", announce({ protocols: { min: 3, max: 1 } })],
    ["fractional versions", announce({ protocols: { min: 1.5, max: 2 } })],
    ["zero versions", announce({ protocols: { min: 0, max: 1 } })],
    ["string versions", announce({ protocols: { min: "1", max: "1" } })],
  ];
  for (const [name, data] of malformed) {
    it(`malformed frame: ${name}`, async () => {
      const env = await loadSdk({ answerDiscover: false });
      env.win.deliver({ data });
      const s = settle(env.sdk.status());
      await advance();
      expect(expectValue(s.outcome()).extension.installed).toBe(false);
      expect(env.script.requests).toEqual([]);
    });
  }

  it("a forged announcement cannot replace a real one", async () => {
    const env = await loadSdk();
    env.script.announce();
    env.win.deliver({ data: announce(), source: new ForeignWindow() });
    env.win.deliver({ data: announce(), origin: "https://evil.example" });
    env.script.autoReply = () => [pageStatus()];
    const s = settle(env.sdk.status());
    await flush();
    expect(expectValue(s.outcome()).extension.version).toBe("1.4.2");
  });

  it("the SDK's own posts looped back to its window are not mistaken for extension traffic", async () => {
    const env = await loadSdk({ answerDiscover: false });
    const s = settle(env.sdk.status());
    await advance();
    expect(expectValue(s.outcome()).extension.installed).toBe(false);
  });
});

describe("replies from anyone but the content script are ignored", () => {
  it("wrong source window, wrong origin, wrong tag: the request stays pending", async () => {
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env);
    const evil = needDigest(1, { certificate: wireCertificate({ displayName: "Forged" }) });
    env.win.deliver({ data: frame(id, evil), source: new ForeignWindow() });
    env.win.deliver({ data: frame(id, evil), origin: "https://evil.example" });
    env.win.deliver({ data: frame(id, evil, { source: "websign-page" }) });
    env.win.deliver({ data: frame(id, evil, { source: undefined }) });
    await flush();
    expect(prepare).not.toHaveBeenCalled();
    expect(digestsSent(env)).toEqual([]);
    expect(s.outcome()).toBeUndefined();
  });

  it("a forged final reply cannot settle a pending request", async () => {
    const env = await loadSdk();
    const s = settle(env.sdk.certificates());
    await flush();
    const id = env.script.only("choose").id;
    const reply = { type: "choose.result", certificates: [wireCertificate()] };
    env.win.deliver({ data: frame(id, reply), source: new ForeignWindow() });
    env.win.deliver({ data: frame(id, reply), origin: "https://evil.example" });
    await flush();
    expect(s.outcome()).toBeUndefined();
    env.script.reply(id, reply);
    await flush();
    expect(expectValue(s.outcome())).toHaveLength(1);
  });

  const malformed: [string, unknown][] = [
    ["null", null],
    ["a string", "x"],
    ["no kind", { source: "websign-extension", id: "p1.aaaaaa", message: { type: "status" } }],
    ["kind discover echoed back", { source: "websign-extension", kind: "discover" }],
    ["message without id", { source: "websign-extension", kind: "message", message: pageStatus() }],
    ["numeric id", { source: "websign-extension", kind: "message", id: 1, message: pageStatus() }],
    [
      "message not an object",
      { source: "websign-extension", kind: "message", id: "IDX", message: "status" },
    ],
    [
      "message without type",
      { source: "websign-extension", kind: "message", id: "IDX", message: {} },
    ],
    [
      "unknown message type",
      { source: "websign-extension", kind: "message", id: "IDX", message: { type: "hello" } },
    ],
    [
      "a request-only type",
      {
        source: "websign-extension",
        kind: "message",
        id: "IDX",
        message: { type: "sign.digest", seq: 1, digest: "AA==" },
      },
    ],
    [
      "an app-internal type",
      { source: "websign-extension", kind: "message", id: "IDX", message: { type: "done" } },
    ],
    [
      "prototype-polluting id",
      { source: "websign-extension", kind: "message", id: "__proto__", message: pageStatus() },
    ],
  ];
  for (const [name, data] of malformed) {
    it(`malformed frame is dropped: ${name}`, async () => {
      const env = await loadSdk();
      const s = settle(env.sdk.certificates());
      await flush();
      const id = env.script.only("choose").id;
      const patched = JSON.parse(JSON.stringify(data ?? null), (_k, v) => (v === "IDX" ? id : v));
      env.win.deliver({ data: patched });
      await flush();
      expect(s.outcome()).toBeUndefined();
    });
  }

  it("unknown ids are ignored without side effects", async () => {
    const env = await loadSdk();
    const { s, prepare } = await beginSign(env);
    env.script.reply("p42.qqqqqq", needDigest(1));
    await flush();
    expect(prepare).not.toHaveBeenCalled();
    expect(s.outcome()).toBeUndefined();
  });

  it("one malformed frame does not break later valid ones", async () => {
    const env = await loadSdk();
    const s = settle(env.sdk.certificates());
    await flush();
    const id = env.script.only("choose").id;
    env.win.deliver({ data: { source: "websign-extension", kind: "message", id, message: null } });
    env.win.deliver({ data: "garbage" });
    env.script.reply(id, { type: "choose.result", certificates: [wireCertificate()] });
    await flush();
    expect(expectValue(s.outcome())).toHaveLength(1);
  });

  it("origin is compared with the page's own origin, whatever it is", async () => {
    const env = await loadSdk({ origin: "https://app.example:8443" });
    env.script.announce();
    const s = settle(env.sdk.certificates());
    await flush();
    const id = env.script.only("choose").id;
    const reply = { type: "choose.result", certificates: [wireCertificate()] };
    env.win.deliver({ data: frame(id, reply), origin: "https://app.example" });
    await flush();
    expect(s.outcome()).toBeUndefined();
    env.script.reply(id, reply);
    await flush();
    expect(s.outcome()?.ok).toBe(true);
    expect(env.win.posted.every((p) => p.targetOrigin === "https://app.example:8443")).toBe(true);
  });
});

async function advance(): Promise<void> {
  await vi.advanceTimersByTimeAsync(1000);
}
