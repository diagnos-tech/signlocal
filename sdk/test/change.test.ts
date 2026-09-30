import { describe, expect, it, vi } from "vitest";
import { loadSdk, settle, useFakeEnvironment } from "./helpers/env";
import type { RequestFrame } from "./helpers/fake-script";
import { pageStatus, wireError } from "./helpers/fixtures";

useFakeEnvironment();

const settleTime = () => vi.advanceTimersByTimeAsync(300);

function replyStatusAndFail(code: string) {
  return (request: RequestFrame) =>
    request.message.type === "status" ? [pageStatus()] : [wireError(code)];
}

describe("onChange()", () => {
  it("does not call the listener on subscription", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [pageStatus()];
    const listener = vi.fn();
    env.sdk.onChange(listener);
    await settleTime();
    expect(listener).not.toHaveBeenCalled();
  });

  it("calls the listener with a fresh Status when the extension announces", async () => {
    const env = await loadSdk({ version: "1.4.2" });
    env.script.autoReply = () => [pageStatus({ remembered: true })];
    const listener = vi.fn();
    env.sdk.onChange(listener);
    env.script.announce();
    await settleTime();
    expect(listener).toHaveBeenCalled();
    expect(listener.mock.calls[0]?.[0]).toMatchObject({
      extension: { installed: true, version: "1.4.2" },
      remembered: true,
      ready: true,
    });
  });

  it("notifies when the extension is installed or updated after the first miss", async () => {
    const env = await loadSdk({ answerDiscover: false });
    const listener = vi.fn();
    env.sdk.onChange(listener);
    const first = settle(env.sdk.status());
    await vi.advanceTimersByTimeAsync(1000);
    expect(first.outcome()?.ok).toBe(true);
    env.script.autoReply = () => [pageStatus()];
    env.script.announce();
    await settleTime();
    expect(listener).toHaveBeenCalled();
    expect(listener.mock.calls.at(-1)?.[0].ready).toBe(true);
  });

  for (const code of ["AppMissing", "AppOutdated"]) {
    it(`re-checks status when a request fails with ${code}`, async () => {
      const env = await loadSdk();
      env.script.announce();
      env.script.autoReply = replyStatusAndFail(code);
      const listener = vi.fn();
      env.sdk.onChange(listener);
      settle(env.sdk.certificates());
      await settleTime();
      expect(listener).toHaveBeenCalledTimes(1);
    });
  }

  it("debounces a burst of failures into one notification (250 ms)", async () => {
    const env = await loadSdk();
    env.script.announce();
    env.script.autoReply = replyStatusAndFail("AppMissing");
    const listener = vi.fn();
    env.sdk.onChange(listener);
    settle(env.sdk.certificates());
    settle(env.sdk.certificates());
    settle(env.sdk.certificates());
    await vi.advanceTimersByTimeAsync(100);
    expect(listener).not.toHaveBeenCalled();
    await settleTime();
    expect(listener).toHaveBeenCalledTimes(1);
  });

  it("other failures do not notify", async () => {
    const env = await loadSdk();
    env.script.announce();
    env.script.autoReply = replyStatusAndFail("UserCancelled");
    const listener = vi.fn();
    env.sdk.onChange(listener);
    settle(env.sdk.certificates());
    await settleTime();
    expect(listener).not.toHaveBeenCalled();
  });

  it("unsubscribe stops notifications and is idempotent", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [pageStatus()];
    const a = vi.fn();
    const b = vi.fn();
    const stopA = env.sdk.onChange(a);
    env.sdk.onChange(b);
    stopA();
    expect(() => stopA()).not.toThrow();
    env.script.announce();
    await settleTime();
    expect(a).not.toHaveBeenCalled();
    expect(b).toHaveBeenCalled();
  });

  it("unsubscribing twice does not remove another subscription of the same function", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [pageStatus()];
    const listener = vi.fn();
    const first = env.sdk.onChange(listener);
    env.sdk.onChange(listener);
    first();
    first();
    env.script.announce();
    await settleTime();
    expect(listener).toHaveBeenCalled();
  });

  it("a pending debounce is dropped after unsubscribe", async () => {
    const env = await loadSdk();
    env.script.announce();
    env.script.autoReply = replyStatusAndFail("AppMissing");
    const listener = vi.fn();
    const stop = env.sdk.onChange(listener);
    settle(env.sdk.certificates());
    await vi.advanceTimersByTimeAsync(50);
    stop();
    await settleTime();
    expect(listener).not.toHaveBeenCalled();
  });
});
