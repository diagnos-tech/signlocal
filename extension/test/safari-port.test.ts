import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { FakeAppex } from "./fakes/appex";
import { fakeBrowser } from "./fakes/browser";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));
vi.mock("../src/background/browser", () => ({
  detectBrowser: async () => ({ name: "safari", version: "18.0" }),
}));

type Module = typeof import("../src/background/connection");
let mod: Module;
let appex: FakeAppex;
const settle = () => vi.advanceTimersByTimeAsync(0);
const choose = (id: string) =>
  ({
    v: 1,
    id,
    type: "choose",
    web: { origin: "https://a.example", topOrigin: "https://a.example" },
  }) as const;

beforeEach(async () => {
  vi.useFakeTimers();
  fakeBrowser.reset();
  fakeBrowser.urlScheme = "safari-web-extension";
  appex = new FakeAppex();
  fakeBrowser.runtime.sendNativeMessage.mockImplementation(appex.handle);
  vi.resetModules();
  mod = await import("../src/background/connection");
});

afterEach(() => {
  vi.useRealTimers();
});

describe("Safari transport: happy path", () => {
  it("opens a session, says hello through it and never touches connectNative", async () => {
    const conn = await mod.connect();
    expect(conn.hello.protocol).toBe(1);
    expect(fakeBrowser.runtime.connectNative).not.toHaveBeenCalled();
    expect(appex.ops().slice(0, 2)).toEqual(["open", "send"]);
    expect(appex.session.received[0]).toMatchObject({ type: "hello", browser: { name: "safari" } });
  });

  it("delivers what the host says later through a single outstanding poll", async () => {
    const conn = await mod.connect();
    const heard = vi.fn();
    conn.onMessage(heard);
    conn.send(choose("5.0.p1"));
    await vi.advanceTimersByTimeAsync(12_000);
    appex.say(appex.session, { v: 1, id: "5.0.p1", type: "choose.result", certificates: [] });
    await settle();
    expect(heard).toHaveBeenCalledWith(
      expect.objectContaining({ id: "5.0.p1", type: "choose.result" }),
    );
    expect(appex.maxParked).toBe(1);
  });

  it("sends messages posted before the session opened, in order, after it opens", async () => {
    const conn = await mod.connect();
    conn.send(choose("5.0.a"));
    conn.send({ v: 1, id: "5.0.a", type: "cancel" });
    await settle();
    expect(appex.session.received.map((m) => m.type)).toEqual(["hello", "choose", "cancel"]);
  });

  it("closes the session when the connection idles out, and stops polling", async () => {
    await mod.connect();
    const id = appex.session.id;
    await vi.advanceTimersByTimeAsync(60_000);
    expect(appex.requests.at(-1)).toEqual({ relay: 1, op: "close", session: id });
    const count = appex.requests.length;
    await vi.advanceTimersByTimeAsync(30_000);
    expect(appex.requests).toHaveLength(count);
  });
});

describe("Safari transport: failures map like a native port", () => {
  it("host exit after hello: the connection closes as heard (Internal for open requests)", async () => {
    const conn = await mod.connect();
    appex.exit(appex.session);
    await settle();
    await expect(conn.closed).resolves.toMatchObject({ heard: true });
  });

  it("host exit before hello: AppMissing", async () => {
    appex.answer = () => [];
    const pending = mod.connect();
    const failure = expect(pending).rejects.toMatchObject({ code: "AppMissing" });
    await settle();
    appex.exit(appex.session);
    await failure;
  });

  it("a poll the appex never answers ends the port and releases the session", async () => {
    const conn = await mod.connect();
    const id = appex.session.id;
    fakeBrowser.runtime.sendNativeMessage.mockImplementation(async (app, request) =>
      (request as { op: string }).op === "poll"
        ? new Promise(() => {})
        : appex.handle(app, request),
    );
    await vi.advanceTimersByTimeAsync(5_000); // the parked poll ends; the next one hangs
    await vi.advanceTimersByTimeAsync(10_000);
    await expect(conn.closed).resolves.toMatchObject({ heard: true });
    expect(appex.requests.at(-1)).toEqual({ relay: 1, op: "close", session: id });
  });

  it("session limit reached (TooManySessions): Internal", async () => {
    appex.openReply = { relay: 1, error: "TooManySessions" };
    await expect(mod.connect()).rejects.toMatchObject({ code: "Internal" });
  });

  it("bundled host missing (HostMissing): AppMissing", async () => {
    appex.openReply = { relay: 1, error: "HostMissing" };
    await expect(mod.connect()).rejects.toMatchObject({ code: "AppMissing" });
  });

  it("appex missing (sendNativeMessage rejects): AppMissing", async () => {
    fakeBrowser.runtime.sendNativeMessage.mockRejectedValue(new Error("no native app"));
    await expect(mod.connect()).rejects.toMatchObject({ code: "AppMissing" });
  });

  it.each([
    [
      "an unknown key",
      { relay: 1, session: "00000000-0000-4000-8000-00000000000A", messages: [], open: true, x: 1 },
    ],
    [
      "a lower-case session",
      { relay: 1, session: "00000000-0000-4000-8000-00000000000a", messages: [], open: true },
    ],
    ["an unknown error", { relay: 1, error: "Nope" }],
    [
      "a non-envelope message",
      { relay: 1, session: "00000000-0000-4000-8000-00000000000A", messages: [1], open: true },
    ],
    ["another relay version", { relay: 2, error: "BadRequest" }],
    ["nothing", undefined],
  ])("a malformed reply (%s): Internal", async (_name, reply) => {
    fakeBrowser.runtime.sendNativeMessage.mockImplementation(async (app, request) =>
      (request as { op: string }).op === "open" ? reply : appex.handle(app, request),
    );
    await expect(mod.connect()).rejects.toMatchObject({ code: "Internal" });
  });

  it("a reply naming another session ends the port and releases ours", async () => {
    const conn = await mod.connect();
    const { id } = appex.session;
    fakeBrowser.runtime.sendNativeMessage.mockResolvedValueOnce({
      relay: 1,
      session: "00000000-0000-4000-8000-0000000000FF",
      messages: [],
      open: true,
    });
    conn.send(choose("5.0.p1"));
    await expect(conn.closed).resolves.toMatchObject({ heard: true });
    expect(appex.requests.at(-1)).toMatchObject({ op: "close", session: id });
  });

  it("NoSession (the appex forgot us): the port closes", async () => {
    const conn = await mod.connect();
    appex.sessions.clear();
    conn.send(choose("5.0.p1"));
    await expect(conn.closed).resolves.toMatchObject({ heard: true });
  });
});

describe("Safari transport: bounded polling", () => {
  it("backs off when the relay answers polls at once with nothing", async () => {
    await mod.connect();
    const { id } = appex.session;
    fakeBrowser.runtime.sendNativeMessage.mockImplementation(async (app, request) =>
      (request as { op: string }).op === "poll"
        ? { relay: 1, session: id, messages: [], open: true }
        : appex.handle(app, request),
    );
    const before = fakeBrowser.runtime.sendNativeMessage.mock.calls.length;
    await vi.advanceTimersByTimeAsync(20_000);
    const polls = fakeBrowser.runtime.sendNativeMessage.mock.calls.length - before;
    expect(polls).toBeLessThan(12);
  });

  it("Chromium and Firefox keep connectNative", async () => {
    fakeBrowser.urlScheme = "moz-extension";
    void mod.connect();
    await settle();
    expect(fakeBrowser.runtime.connectNative).toHaveBeenCalledTimes(1);
    expect(fakeBrowser.runtime.sendNativeMessage).not.toHaveBeenCalled();
  });
});
