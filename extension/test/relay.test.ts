import { beforeEach, describe, expect, it, vi } from "vitest";
import { startRelay } from "../src/content/relay";
import { fakeBrowser } from "./fakes/browser";
import { type FakeWindow, installWindow } from "./fakes/window";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));

const ORIGIN = "https://app.example.com";
const request = (over: Record<string, unknown> = {}) => ({
  source: "websign-page",
  kind: "request",
  id: "p1",
  message: { type: "sign.begin", hash: "SHA-256" },
  ...over,
});

let win: FakeWindow;
const flush = () => new Promise((resolve) => setTimeout(resolve, 0));
const relayed = () =>
  fakeBrowser.runtime.sendMessage.mock.calls.map((call) => call[0] as Record<string, unknown>);
const UUID = /^[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}$/;
const own = { id: fakeBrowser.runtime.id };

/** This document's token, learned from a relayed request as the background would. */
async function documentToken(): Promise<string> {
  win.dispatchMessage(request({ id: "probe", message: { type: "status" } }));
  await flush();
  return relayed().at(-1)?.document as string;
}

/** Delivers a background → content message, as `tabs.sendMessage` would. */
function fromBackground(message: unknown, sender: unknown = own): void {
  fakeBrowser.runtime.onMessage.emit(message, sender, () => {});
}

beforeEach(() => {
  fakeBrowser.reset();
  win = installWindow(ORIGIN);
  startRelay();
});

describe("relay: what is forwarded", () => {
  it("forwards a well-formed request to the background with the page id and message", async () => {
    win.dispatchMessage(request());
    await flush();
    expect(relayed()).toEqual([
      {
        kind: "websign-request",
        document: expect.stringMatching(UUID),
        id: "p1",
        message: { type: "sign.begin", hash: "SHA-256" },
      },
    ]);
  });

  it("forwards status, choose with filter, and cancel", async () => {
    win.dispatchMessage(request({ id: "a", message: { type: "status" } }));
    win.dispatchMessage(
      request({ id: "b", message: { type: "choose", filter: { algorithms: ["ECDSA"] } } }),
    );
    win.dispatchMessage(request({ id: "c", message: { type: "cancel" } }));
    await flush();
    expect(relayed()).toHaveLength(3);
  });

  it("accepts an id of exactly 40 characters", async () => {
    win.dispatchMessage(request({ id: "x".repeat(40) }));
    await flush();
    expect(relayed()).toHaveLength(1);
  });
});

describe("relay: what is refused", () => {
  it("ignores messages from another window", async () => {
    win.dispatchMessage(request(), { source: {} });
    win.dispatchMessage(request(), { source: null });
    await flush();
    expect(relayed()).toEqual([]);
  });

  it("ignores messages whose event origin differs from the page origin", async () => {
    win.dispatchMessage(request(), { origin: "https://evil.example" });
    await flush();
    expect(relayed()).toEqual([]);
  });

  it.each([
    ["other source", request({ source: "someone-else" })],
    ["extension source (echo)", request({ source: "websign-extension" })],
    ["unknown kind", request({ kind: "hello" })],
    ["missing kind", request({ kind: undefined })],
    ["numeric id", request({ id: 7 })],
    ["missing id", request({ id: undefined })],
    ["41 char id", request({ id: "x".repeat(41) })],
    ["missing message", request({ message: undefined })],
    ["array message", request({ message: [] })],
    ["string message", request({ message: "status" })],
    ["null data", null],
    ["string data", "websign-page"],
    ["array data", [request()]],
  ])("does not relay: %s", async (_name, data) => {
    win.dispatchMessage(data);
    await flush();
    expect(relayed()).toEqual([]);
  });

  it("does not relay a string field longer than 256 characters", async () => {
    win.dispatchMessage(
      request({ message: { type: "sign.digest", seq: 1, digest: "A".repeat(300) } }),
    );
    await flush();
    expect(relayed()).toEqual([]);
  });

  it("answers a malformed request itself with InvalidRequest, to the page's origin", async () => {
    win.dispatchMessage(request({ message: { type: "sign.begin", hash: "MD5" } }));
    await flush();
    expect(relayed()).toEqual([]);
    expect(win.posted).toEqual([
      {
        targetOrigin: ORIGIN,
        message: {
          source: "websign-extension",
          kind: "message",
          id: "p1",
          message: { type: "error", code: "InvalidRequest", message: expect.any(String) },
        },
      },
    ]);
  });

  it("answers discover with an announcement instead of relaying it", async () => {
    win.dispatchMessage({ source: "websign-page", kind: "discover" });
    await flush();
    expect(relayed()).toEqual([]);
    expect(win.fromExtension()).toMatchObject([{ kind: "announce" }]);
  });

  it("ignores a discover from another window", async () => {
    win.dispatchMessage({ source: "websign-page", kind: "discover" }, { source: {} });
    await flush();
    expect(win.posted).toEqual([]);
  });
});

describe("relay: the page cannot forge the browser's facts", () => {
  it("drops envelope fields the page adds (origin, web, tab, frame, document)", async () => {
    win.dispatchMessage(
      request({
        origin: "https://evil.example",
        topOrigin: "https://evil.example",
        web: { origin: "https://evil.example", topOrigin: "https://evil.example" },
        tabId: 99,
        frameId: 42,
        document: "00000000-0000-4000-8000-000000000000",
      }),
    );
    await flush();
    expect(relayed()).toHaveLength(1);
    const text = JSON.stringify(relayed()[0]);
    for (const forged of ["evil.example", "tabId", "frameId", "00000000-0000"]) {
      expect(text).not.toContain(forged);
    }
  });

  it("refuses a request carrying web or unknown fields instead of relaying it", async () => {
    const web = { origin: "https://evil.example", topOrigin: "https://evil.example" };
    win.dispatchMessage(request({ message: { type: "sign.begin", hash: "SHA-256", web } }));
    win.dispatchMessage(request({ id: "p2", message: { type: "status", nested: { a: 1 } } }));
    await flush();
    expect(relayed()).toEqual([]);
    expect(win.fromExtension()).toMatchObject([
      { id: "p1", message: { code: "InvalidRequest" } },
      { id: "p2", message: { code: "InvalidRequest" } },
    ]);
  });
});

describe("relay: replies", () => {
  const error = { type: "error", code: "UserCancelled", message: "cancelled" };

  it("posts a background reply to the page with source, id and the page's own origin", async () => {
    const document = await documentToken();
    fromBackground({ kind: "websign-reply", document, id: "p1", message: error });
    expect(win.posted).toEqual([
      {
        targetOrigin: ORIGIN,
        message: { source: "websign-extension", kind: "message", id: "p1", message: error },
      },
    ]);
  });

  it("drops a reply meant for another document of the same frame", async () => {
    await documentToken();
    fromBackground({
      kind: "websign-reply",
      document: "00000000-0000-4000-8000-000000000000",
      id: "p1",
      message: error,
    });
    expect(win.posted).toEqual([]);
  });

  it("drops messages that do not come from this extension", async () => {
    const document = await documentToken();
    fromBackground({ kind: "websign-reply", document, id: "p1", message: error }, { id: "other" });
    fromBackground({ kind: "websign-reply", document, id: "p1", message: error }, {});
    expect(win.posted).toEqual([]);
  });

  it("answers the background's top-origin question from the top frame only", () => {
    const respond = vi.fn();
    Object.assign(win, { top: win });
    fakeBrowser.runtime.onMessage.emit({ kind: "websign-origin" }, own, respond);
    expect(respond).toHaveBeenCalledWith(ORIGIN);
    respond.mockClear();
    Object.assign(win, { top: {} });
    fakeBrowser.runtime.onMessage.emit({ kind: "websign-origin" }, own, respond);
    expect(respond).not.toHaveBeenCalled();
  });

  it("tells the background when the document goes away, with the same token", async () => {
    const document = await documentToken();
    win.dispatch("pagehide");
    expect(relayed().at(-1)).toEqual({ kind: "websign-gone", document });
  });

  it("never posts to the wildcard origin", async () => {
    const document = await documentToken();
    fromBackground({ kind: "websign-reply", document, id: "p1", message: { type: "status" } });
    win.dispatchMessage(request({ message: "bad" }));
    await flush();
    expect(win.posted.length).toBeGreaterThan(0);
    expect(win.posted.every((p) => p.targetOrigin === ORIGIN)).toBe(true);
  });
});
