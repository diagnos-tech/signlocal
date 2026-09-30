import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { NATIVE_HOST } from "../src/generated/project";
import { type FakePort, fakeBrowser } from "./fakes/browser";
import { helloEnvelope } from "./fakes/protocol";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));
vi.mock("../src/background/browser", () => ({
  detectBrowser: async () => ({ name: "chrome", version: "121.0" }),
}));

type Module = typeof import("../src/background/connection");
let mod: Module;
const settle = () => vi.advanceTimersByTimeAsync(0);
const hello = (port: FakePort) => port.sent[0] as { id: string } & Record<string, unknown>;

/** Starts `connect()` and answers the hello like the app would. */
async function connectAnswered(version = "1.4.0") {
  const pending = mod.connect();
  await settle();
  const port = fakeBrowser.ports.at(-1) as FakePort;
  port.receive(helloEnvelope(hello(port).id, version));
  return { connection: await pending, port };
}

beforeEach(async () => {
  vi.useFakeTimers();
  fakeBrowser.reset();
  vi.resetModules();
  mod = await import("../src/background/connection");
});

afterEach(() => {
  vi.useRealTimers();
});

describe("connect: opening", () => {
  it("opens the native host once and says hello first", async () => {
    const { port } = await connectAnswered();
    expect(fakeBrowser.runtime.connectNative).toHaveBeenCalledTimes(1);
    expect(fakeBrowser.runtime.connectNative).toHaveBeenCalledWith(NATIVE_HOST);
    expect(port.sent).toHaveLength(1);
    expect(port.sent[0]).toMatchObject({
      v: 1,
      type: "hello",
      client: { name: "websign-extension", version: fakeBrowser.manifestVersion },
      protocols: { min: 1, max: 1 },
      browser: { name: "chrome", version: "121.0", reason: "page" },
    });
  });

  it.each(["startup", "installed", "popup"] as const)(
    "tells the app why it connected (%s)",
    async (reason) => {
      void mod.connect(reason);
      await settle();
      expect(fakeBrowser.ports[0]?.sent[0]).toMatchObject({ browser: { reason } });
    },
  );

  it("exposes the hello reply", async () => {
    const { connection } = await connectAnswered("1.4.0");
    expect(connection.hello).toMatchObject({ protocol: 1, app: { version: "1.4.0" } });
  });

  it("shares one port between concurrent and later callers", async () => {
    const a = mod.connect();
    const b = mod.connect();
    await settle();
    expect(fakeBrowser.ports).toHaveLength(1);
    const port = fakeBrowser.ports[0] as FakePort;
    port.receive(helloEnvelope(hello(port).id));
    const [ca, cb] = await Promise.all([a, b]);
    const later = await mod.connect();
    expect(fakeBrowser.runtime.connectNative).toHaveBeenCalledTimes(1);
    expect(cb).toBe(ca);
    expect(later).toBe(ca);
  });
});

describe("connect: failures", () => {
  it("AppMissing when the port closes before any message (host not found)", async () => {
    const pending = mod.connect();
    const failure = expect(pending).rejects.toMatchObject({ code: "AppMissing" });
    await settle();
    (fakeBrowser.ports[0] as FakePort).drop();
    await failure;
  });

  it("AppMissing when connectNative itself throws", async () => {
    // Firefox may throw synchronously for a missing host: the same "host not found".
    fakeBrowser.connectNativeError = new Error("No such native application dev.websign.host");
    await expect(mod.connect()).rejects.toMatchObject({ code: "AppMissing" });
  });

  it("Internal when the app stays silent for 3 s", async () => {
    const pending = mod.connect();
    const failure = expect(pending).rejects.toMatchObject({ code: "Internal" });
    await vi.advanceTimersByTimeAsync(2999);
    await settle();
    expect(fakeBrowser.ports[0]?.disconnect).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    await failure;
  });

  it("opens a fresh port after a failed attempt", async () => {
    const first = mod.connect();
    const failure = expect(first).rejects.toBeDefined();
    await settle();
    (fakeBrowser.ports[0] as FakePort).drop();
    await failure;
    const { port } = await connectAnswered();
    expect(fakeBrowser.ports).toHaveLength(2);
    expect(port).toBe(fakeBrowser.ports[1]);
  });
});

describe("connection: traffic", () => {
  it("posts client messages to the port", async () => {
    const { connection, port } = await connectAnswered();
    connection.send({
      v: 1,
      id: "1.0.p1",
      type: "status",
      web: { origin: "https://a.example", topOrigin: "https://a.example" },
    });
    expect(port.sent.at(-1)).toMatchObject({ id: "1.0.p1", type: "status" });
  });

  it("delivers app messages to listeners until they unsubscribe", async () => {
    const { connection, port } = await connectAnswered();
    const listener = vi.fn();
    const off = connection.onMessage(listener);
    port.receive({ v: 1, id: "1.0.p1", type: "done" });
    off();
    port.receive({ v: 1, id: "1.0.p2", type: "done" });
    expect(listener).toHaveBeenCalledTimes(1);
    expect(listener).toHaveBeenCalledWith({ v: 1, id: "1.0.p1", type: "done" });
  });

  it("does not hand the hello reply to message listeners twice", async () => {
    const pending = mod.connect();
    await settle();
    const port = fakeBrowser.ports[0] as FakePort;
    port.receive(helloEnvelope(hello(port).id));
    const connection = await pending;
    const listener = vi.fn();
    connection.onMessage(listener);
    expect(listener).not.toHaveBeenCalled();
  });
});

describe("connection: closing", () => {
  it("closed reports heard=true when the app spoke before the port closed", async () => {
    const { connection, port } = await connectAnswered();
    port.drop();
    await expect(connection.closed).resolves.toMatchObject({ heard: true });
  });

  it("closed carries a reason string", async () => {
    const { connection, port } = await connectAnswered();
    port.drop();
    const { reason } = await connection.closed;
    expect(typeof reason).toBe("string");
  });

  it("reconnects with a new port after the old one closed", async () => {
    const first = await connectAnswered();
    first.port.drop();
    await first.connection.closed;
    const second = await connectAnswered();
    expect(second.connection).not.toBe(first.connection);
    expect(fakeBrowser.runtime.connectNative).toHaveBeenCalledTimes(2);
  });

  it("closes the port after 60 s without traffic", async () => {
    const { port } = await connectAnswered();
    await vi.advanceTimersByTimeAsync(59_999);
    expect(port.disconnect).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(port.disconnect).toHaveBeenCalledTimes(1);
  });

  it("stays open while a request is unanswered, then idles out after its final reply", async () => {
    const { connection, port } = await connectAnswered();
    connection.send({
      v: 1,
      id: "1.0.p1",
      type: "choose",
      web: { origin: "https://a.example", topOrigin: "https://a.example" },
    });
    await vi.advanceTimersByTimeAsync(5 * 60_000);
    expect(port.disconnect).not.toHaveBeenCalled();
    port.receive({ v: 1, id: "1.0.p1", type: "choose.result", certificates: [] });
    await vi.advanceTimersByTimeAsync(59_999);
    expect(port.disconnect).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(1);
    expect(port.disconnect).toHaveBeenCalledTimes(1);
  });

  it("traffic restarts the idle countdown", async () => {
    const { connection, port } = await connectAnswered();
    await vi.advanceTimersByTimeAsync(40_000);
    connection.send({ v: 1, id: "1.0.p1", type: "cancel" });
    await vi.advanceTimersByTimeAsync(40_000);
    expect(port.disconnect).not.toHaveBeenCalled();
    await vi.advanceTimersByTimeAsync(20_000);
    expect(port.disconnect).toHaveBeenCalledTimes(1);
  });
});
