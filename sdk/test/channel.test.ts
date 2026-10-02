import { describe, expect, it, vi } from "vitest";
import { loadSdk, settle, useFakeEnvironment } from "./helpers/env";
import { expectError } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { needDigest, pageStatus, signResult, wireError } from "./helpers/fixtures";

useFakeEnvironment();

async function announced() {
  const env = await loadSdk();
  env.script.announce();
  return env;
}

describe("channel.send()", () => {
  it("posts {source, kind:'request', id, message} to its own origin", async () => {
    const env = await announced();
    const { id } = env.channel.send({ type: "status" }, () => {});
    await flush();
    expect(env.win.posted).toHaveLength(1);
    expect(env.win.posted[0]).toEqual({
      data: { source: "websign-page", kind: "request", id, message: { type: "status" } },
      targetOrigin: env.win.origin,
    });
  });

  it("ids look like p<counter>.<6 base36 chars>, are valid protocol ids, and never repeat", async () => {
    const env = await announced();
    const ids = Array.from({ length: 20 }, () => env.channel.send({ type: "status" }, () => {}).id);
    for (const id of ids) {
      expect(id).toMatch(/^p\d+\.[0-9a-z]{6}$/);
      expect(id).toMatch(/^[A-Za-z0-9._:-]{1,64}$/);
    }
    expect(new Set(ids).size).toBe(20);
    const counters = ids.map((id) => Number(id.slice(1).split(".")[0]));
    expect(counters).toEqual([...counters].sort((a, b) => a - b));
    expect(new Set(counters).size).toBe(20);
  });

  it("suffixes are random, not derived from the counter", async () => {
    const env = await announced();
    const ids = Array.from({ length: 10 }, () => env.channel.send({ type: "status" }, () => {}).id);
    expect(new Set(ids.map((id) => id.split(".")[1])).size).toBeGreaterThan(1);
  });

  it("delivers every reply with its id, and done resolves only after a final one", async () => {
    const env = await announced();
    const replies: unknown[] = [];
    const { id, done } = env.channel.send({ type: "sign.begin", hash: "SHA-256" }, (r) =>
      replies.push(r),
    );
    const d = settle(done);
    await flush();
    env.script.reply(id, needDigest(1));
    env.script.reply(id, needDigest(2));
    await flush();
    expect(replies).toHaveLength(2);
    expect(d.outcome()).toBeUndefined();
    env.script.reply(id, signResult());
    await flush();
    expect(d.outcome()?.ok).toBe(true);
    expect(replies).toHaveLength(3);
  });

  for (const [type, reply] of [
    ["status", pageStatus()],
    ["choose.result", { type: "choose.result", certificates: [] }],
    ["sign.result", signResult()],
    ["error", wireError("Busy")],
  ] as const) {
    it(`${type} is a final reply`, async () => {
      const env = await announced();
      const { id, done } = env.channel.send({ type: "status" }, () => {});
      const d = settle(done);
      env.script.reply(id, reply);
      await flush();
      expect(d.outcome()?.ok).toBe(true);
    });
  }

  it("stops delivering after the final reply", async () => {
    const env = await announced();
    const onReply = vi.fn();
    const { id } = env.channel.send({ type: "status" }, onReply);
    env.script.reply(id, pageStatus());
    env.script.reply(id, pageStatus());
    await flush();
    expect(onReply).toHaveBeenCalledTimes(1);
  });

  it("does not deliver replies for other ids", async () => {
    const env = await announced();
    const onReply = vi.fn();
    env.channel.send({ type: "status" }, onReply);
    env.script.reply("p999.zzzzzz", pageStatus());
    await flush();
    expect(onReply).not.toHaveBeenCalled();
  });

  it("abort posts cancel with the same id and rejects Aborted without a reply", async () => {
    const env = await announced();
    const ctl = new AbortController();
    const onReply = vi.fn();
    const { id, done } = env.channel.send(
      { type: "sign.begin", hash: "SHA-256" },
      onReply,
      ctl.signal,
    );
    const d = settle(done);
    ctl.abort();
    await flush();
    expect(env.script.only("cancel")).toMatchObject({ id, message: { type: "cancel" } });
    expectError(env, d.outcome(), "Aborted");
    env.script.reply(id, wireError("Aborted"));
    await flush();
    expect(onReply).not.toHaveBeenCalled();
  });

  it("an already aborted signal posts nothing and rejects Aborted", async () => {
    const env = await announced();
    const { done } = env.channel.send({ type: "status" }, () => {}, AbortSignal.abort());
    const d = settle(done);
    await flush();
    expectError(env, d.outcome(), "Aborted");
    expect(env.script.requests).toEqual([]);
  });
});

describe("channel.discover()", () => {
  it("resolves with the announcement", async () => {
    const env = await loadSdk({
      version: "2.0.1",
      browser: "firefox",
      protocols: { min: 1, max: 2 },
    });
    const p = env.channel.discover();
    await flush();
    expect(await p).toEqual({
      extension: { version: "2.0.1", browser: "firefox" },
      protocols: { min: 1, max: 2 },
    });
  });

  it("times out with null after DISCOVERY_TIMEOUT_MS", async () => {
    const env = await loadSdk({ answerDiscover: false });
    expect(env.channel.DISCOVERY_TIMEOUT_MS).toBe(1000);
    const p = env.channel.discover();
    await vi.advanceTimersByTimeAsync(env.channel.DISCOVERY_TIMEOUT_MS);
    expect(await p).toBeNull();
  });

  it("a later announcement replaces the cached one", async () => {
    const env = await loadSdk({ version: "1.0.0" });
    const first = env.channel.discover();
    await flush();
    await first;
    env.win.deliver({
      data: { ...env.script.announcement(), extension: { version: "1.1.0", browser: "chrome" } },
    });
    expect((await env.channel.discover())?.extension.version).toBe("1.1.0");
  });
});
