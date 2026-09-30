import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PageReply, PageRequest } from "../src/generated";
import { fakeBrowser } from "./fakes/browser";
import { FakeConnection } from "./fakes/connection";
import { appInfo } from "./fakes/protocol";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));
vi.mock("../src/background/browser", () => ({
  detectBrowser: async () => ({ name: "chrome", version: "121.0" }),
}));
const connectMock = vi.hoisted(() => vi.fn());
vi.mock("../src/background/connection", () => ({ connect: connectMock }));

const context = { origin: "https://app.example.com", topOrigin: "https://top.example.org" };
const sender = (tabId = 5, frameId = 0) => ({ tabId, frameId, context });
const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

type Router = typeof import("../src/background/router");
let router: Router;
let conn: FakeConnection;
let replies: PageReply[];
const reply = (message: PageReply) => {
  replies.push(message);
};

async function send(request: PageRequest, pageId = "p1", who = sender()) {
  router.route(who, pageId, request, reply);
  await flush();
}

beforeEach(async () => {
  fakeBrowser.reset();
  vi.resetModules();
  conn = new FakeConnection();
  connectMock.mockReset();
  connectMock.mockResolvedValue(conn);
  replies = [];
  router = await import("../src/background/router");
});

describe("route: forwarding", () => {
  it("maps (tab, frame, page id) to a native id and adds the browser's web context", async () => {
    await send({ type: "choose", filter: { algorithms: ["ECDSA"] } }, "p1", sender(5, 3));
    expect(conn.sent).toHaveLength(1);
    expect(conn.sent[0]).toMatchObject({
      v: 1,
      id: "5.3.p1",
      type: "choose",
      web: context,
      filter: { algorithms: ["ECDSA"] },
    });
  });

  it("forwards sign.begin with hash, algorithms and certificate", async () => {
    const certificate = "ab".repeat(32);
    await send({ type: "sign.begin", hash: "SHA-384", algorithms: ["RSASSA-PSS"], certificate });
    expect(conn.sent[0]).toMatchObject({
      id: "5.0.p1",
      type: "sign.begin",
      web: context,
      hash: "SHA-384",
      algorithms: ["RSASSA-PSS"],
      certificate,
    });
  });

  it("forwards sign.digest and cancel under the same native id as the request", async () => {
    await send({ type: "sign.begin", hash: "SHA-256" });
    await send({ type: "sign.digest", seq: 1, digest: `${"A".repeat(43)}=` });
    await send({ type: "cancel" });
    expect(conn.sent.map((m) => [m.type, m.id])).toEqual([
      ["sign.begin", "5.0.p1"],
      ["sign.digest", "5.0.p1"],
      ["cancel", "5.0.p1"],
    ]);
  });

  it("keeps the same page id in different frames or tabs apart", async () => {
    await send({ type: "choose" }, "p1", sender(5, 0));
    await send({ type: "choose" }, "p1", sender(5, 2));
    await send({ type: "choose" }, "p1", sender(6, 0));
    expect(new Set(conn.sent.map((m) => m.id))).toEqual(new Set(["5.0.p1", "5.2.p1", "6.0.p1"]));
  });

  it("refuses a page id longer than 40 characters with InvalidRequest", async () => {
    await send({ type: "choose" }, "x".repeat(41));
    expect(conn.sent).toEqual([]);
    expect(replies).toMatchObject([{ type: "error", code: "InvalidRequest" }]);
  });

  it("accepts a 40 character page id and keeps the native id within 64 characters", async () => {
    await send({ type: "choose" }, "x".repeat(40), sender(123456, 654321));
    expect(conn.sent).toHaveLength(1);
    expect(conn.sent[0]?.id.length).toBeLessThanOrEqual(64);
  });
});

describe("route: replies", () => {
  it("relays app messages for a request back to its reply callback", async () => {
    await send({ type: "sign.begin", hash: "SHA-256" });
    const need = {
      type: "sign.need_digest",
      seq: 1,
      hash: "SHA-256",
      algorithm: "ECDSA",
      certificate: {},
    };
    conn.emit("5.0.p1", need);
    conn.emit("5.0.p1", { type: "error", code: "UserCancelled", message: "no" });
    expect(replies).toMatchObject([need, { type: "error", code: "UserCancelled" }]);
  });

  it("does not deliver messages of other requests", async () => {
    await send({ type: "choose" }, "p1");
    await send({ type: "choose" }, "p2");
    conn.emit("5.0.p2", { type: "choose.result", certificates: [] });
    expect(replies).toMatchObject([{ type: "choose.result" }]);
    expect(replies).toHaveLength(1);
  });

  it("relays the final choose.result", async () => {
    await send({ type: "choose" });
    conn.emit("5.0.p1", { type: "choose.result", certificates: [] });
    expect(replies).toEqual([{ type: "choose.result", certificates: [] }]);
  });

  it("drops the request after its final message (a late message reaches nobody)", async () => {
    await send({ type: "choose" });
    conn.emit("5.0.p1", { type: "choose.result", certificates: [] });
    conn.emit("5.0.p1", { type: "choose.result", certificates: [] });
    expect(replies).toHaveLength(1);
  });
});

describe("route: one shared port, isolated tabs", () => {
  const replyFor = (log: PageReply[]) => (message: PageReply) => {
    log.push(message);
  };

  it("never lets another tab or frame continue, cancel or read a request with the same page id", async () => {
    const tab5: PageReply[] = [];
    const tab6: PageReply[] = [];
    router.route(sender(5, 0), "p1", { type: "sign.begin", hash: "SHA-256" }, replyFor(tab5));
    await flush();
    router.route(
      sender(6, 0),
      "p1",
      { type: "sign.digest", seq: 1, digest: `${"A".repeat(43)}=` },
      replyFor(tab6),
    );
    router.route(sender(5, 1), "p1", { type: "cancel" }, replyFor(tab6));
    expect(conn.sent.map((m) => [m.type, m.id])).toEqual([["sign.begin", "5.0.p1"]]);

    const result = { type: "sign.result", signature: "sig", certificate: {}, chain: [] };
    conn.emit("5.0.p1", result);
    conn.emit("6.0.p1", { type: "choose.result", certificates: [] });
    expect(tab5).toMatchObject([result]);
    expect(tab6).toEqual([]);
  });

  it("ignores app messages for ids no request owns (the extension's own asks included)", async () => {
    await send({ type: "choose" });
    conn.emit("ext.1", { type: "done" });
    conn.emit("5.0", { type: "choose.result", certificates: [] });
    conn.emit("5.0.p1.x", { type: "choose.result", certificates: [] });
    expect(replies).toEqual([]);
  });
});

describe("route: status is answered by the extension", () => {
  it("combines its own facts with the app's status reply", async () => {
    fakeBrowser.manifestVersion = "1.4.2";
    await send({ type: "status" });
    expect(conn.sent).toMatchObject([{ id: "5.0.p1", type: "status", web: context }]);
    conn.emit("5.0.p1", { type: "status", app: appInfo("1.4.0"), remembered: true });
    await flush();
    expect(replies).toHaveLength(1);
    expect(replies[0]).toMatchObject({
      type: "status",
      extension: { version: "1.4.2", browser: "chrome" },
      app: { version: "1.4.0" },
      appOutdated: false,
      remembered: true,
    });
  });
});

describe("route: status when the app stays silent", () => {
  it("answers with the hello's app and remembered false after 1.5 s (inside the SDK's 5 s)", async () => {
    vi.useFakeTimers();
    try {
      router.route(sender(), "p1", { type: "status" }, reply);
      await vi.advanceTimersByTimeAsync(1_499);
      expect(replies).toEqual([]);
      await vi.advanceTimersByTimeAsync(1);
      expect(replies).toMatchObject([
        { type: "status", app: { version: "1.4.0" }, appOutdated: false, remembered: false },
      ]);
      conn.emit("5.0.p1", { type: "status", app: appInfo("1.4.0"), remembered: true });
      await vi.advanceTimersByTimeAsync(0);
      expect(replies).toHaveLength(1);
    } finally {
      vi.useRealTimers();
    }
  });
});
