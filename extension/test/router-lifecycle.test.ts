import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PageReply, PageRequest } from "../src/generated";
import { fakeBrowser } from "./fakes/browser";
import { FakeConnection } from "./fakes/connection";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));
vi.mock("../src/background/browser", () => ({
  detectBrowser: async () => ({ name: "chrome", version: "121.0" }),
}));
const connectMock = vi.hoisted(() => vi.fn());
vi.mock("../src/background/connection", () => ({ connect: connectMock }));

const context = { origin: "https://app.example.com", topOrigin: "https://app.example.com" };
const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

type Router = typeof import("../src/background/router");
let router: Router;
let requests: typeof import("../src/background/requests");
let conn: FakeConnection;
let replies: PageReply[];

async function open(tabId: number, pageId: string, request: PageRequest = { type: "choose" }) {
  router.route({ tabId, frameId: 0, context }, pageId, request, (m) => replies.push(m));
  await flush();
}
const cancelled = () => conn.sentOfType("cancel").map((m) => m.id);

beforeEach(async () => {
  fakeBrowser.reset();
  vi.resetModules();
  conn = new FakeConnection();
  connectMock.mockReset();
  connectMock.mockResolvedValue(conn);
  replies = [];
  router = await import("../src/background/router");
  requests = await import("../src/background/requests");
  (await import("../src/background/tabs")).watchTabs();
});

describe("tab closed", () => {
  it("cancels every open request of that tab and only of that tab", async () => {
    await open(5, "p1");
    await open(5, "p2", { type: "sign.begin", hash: "SHA-256" });
    await open(6, "p1");
    fakeBrowser.tabs.onRemoved.emit(5);
    expect(new Set(cancelled())).toEqual(new Set(["5.0.p1", "5.0.p2"]));
  });

  it("cancels requests of every frame in the tab", async () => {
    router.route({ tabId: 5, frameId: 0, context }, "a", { type: "choose" }, () => {});
    router.route({ tabId: 5, frameId: 9, context }, "a", { type: "choose" }, () => {});
    await flush();
    fakeBrowser.tabs.onRemoved.emit(5);
    expect(new Set(cancelled())).toEqual(new Set(["5.0.a", "5.9.a"]));
  });

  it("does not cancel requests that already finished", async () => {
    await open(5, "p1");
    conn.emit("5.0.p1", { type: "choose.result", certificates: [] });
    fakeBrowser.tabs.onRemoved.emit(5);
    expect(cancelled()).toEqual([]);
  });

  it("cancels once, not again on a second removal event", async () => {
    await open(5, "p1");
    fakeBrowser.tabs.onRemoved.emit(5);
    fakeBrowser.tabs.onRemoved.emit(5);
    expect(cancelled()).toEqual(["5.0.p1"]);
  });
});

describe("top-level navigation", () => {
  const loading = (tabId: number, url: string) =>
    fakeBrowser.tabs.onUpdated.emit(tabId, { status: "loading", url }, {});

  it("cancels the tab's requests when the origin changes while loading", async () => {
    await open(5, "p1");
    await open(6, "p1");
    loading(5, "https://elsewhere.example.net/next");
    expect(cancelled()).toEqual(["5.0.p1"]);
  });

  it("keeps requests on same-origin navigation", async () => {
    await open(5, "p1");
    loading(5, "https://app.example.com/other/path?x=1#y");
    expect(cancelled()).toEqual([]);
  });

  it("compares with the top origin of each request, not of the frame", async () => {
    const framed = { origin: "https://widget.example", topOrigin: context.topOrigin };
    router.route({ tabId: 5, frameId: 3, context: framed }, "w", { type: "choose" }, () => {});
    await flush();
    loading(5, "https://widget.example/");
    expect(cancelled()).toEqual(["5.3.w"]);
  });

  it("cancels on a URL without a parseable origin", async () => {
    await open(5, "p1");
    loading(5, "not a url");
    expect(cancelled()).toEqual(["5.0.p1"]);
  });

  it("ignores updates that are not a loading navigation with a URL", async () => {
    await open(5, "p1");
    fakeBrowser.tabs.onUpdated.emit(5, { status: "complete", url: "https://other.example/" }, {});
    fakeBrowser.tabs.onUpdated.emit(5, { status: "loading" }, {});
    fakeBrowser.tabs.onUpdated.emit(5, {}, {});
    expect(cancelled()).toEqual([]);
  });
});

describe("document gone (pagehide)", () => {
  const docA = "aaaaaaaa-0000-4000-8000-000000000000";
  const docB = "bbbbbbbb-0000-4000-8000-000000000000";
  const openIn = async (frameId: number, document: string, pageId: string) => {
    router.route({ tabId: 5, frameId, context, document }, pageId, { type: "choose" }, () => {});
    await flush();
  };

  it("cancels only that document's requests, not its successor's in the same frame", async () => {
    await openIn(0, docA, "old");
    await openIn(0, docB, "new");
    await openIn(2, docA, "other-frame");
    requests.cancelDocument(5, 0, docA);
    expect(cancelled()).toEqual(["5.0.old"]);
  });
});

describe("port closed", () => {
  it("answers open requests with AppMissing when the app was never heard", async () => {
    // A resolved hello means the app was heard; heard=false still pins the §2.1 mapping.
    await open(5, "p1");
    await open(5, "p2");
    conn.close(false);
    await flush();
    expect(replies).toHaveLength(2);
    for (const r of replies) expect(r).toMatchObject({ type: "error", code: "AppMissing" });
  });

  it("answers open requests with Internal when the app was heard", async () => {
    await open(5, "p1");
    conn.close(true);
    await flush();
    expect(replies).toMatchObject([{ type: "error", code: "Internal" }]);
  });

  it("does not answer requests that already finished", async () => {
    await open(5, "p1");
    conn.emit("5.0.p1", { type: "choose.result", certificates: [] });
    replies.length = 0;
    conn.close(true);
    await flush();
    expect(replies).toEqual([]);
  });
});
