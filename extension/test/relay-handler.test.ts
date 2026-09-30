/**
 * The background's side of the relay: who may send, where the origin comes
 * from (the browser, never the payload) and where replies go.
 */

import { beforeEach, describe, expect, it, vi } from "vitest";
import { fakeBrowser } from "./fakes/browser";
import { FakeConnection } from "./fakes/connection";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));
vi.mock("../src/background/browser", () => ({
  detectBrowser: async () => ({ name: "chrome", version: "121.0" }),
}));
const connectMock = vi.hoisted(() => vi.fn());
vi.mock("../src/background/connection", () => ({ connect: connectMock }));

const DOC = "aaaaaaaa-0000-4000-8000-000000000000";
const PAGE = "https://app.example.com/sign";
const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

let handle: typeof import("../src/background/relay-handler").handleContentMessage;
let conn: FakeConnection;

interface Sender {
  id?: string;
  url?: string;
  frameId?: number;
  tab?: { id?: number; url?: string } | undefined;
}
const tabSender = (over: Partial<Sender> = {}): Sender => ({
  id: fakeBrowser.runtime.id,
  url: PAGE,
  frameId: 0,
  tab: { id: 7, url: PAGE },
  ...over,
});
const request = (message: unknown = { type: "choose" }, over: Record<string, unknown> = {}) => ({
  kind: "websign-request",
  document: DOC,
  id: "p1",
  message,
  ...over,
});
const repliesSent = () => fakeBrowser.tabs.sendMessage.mock.calls;

beforeEach(async () => {
  fakeBrowser.reset();
  vi.resetModules();
  conn = new FakeConnection();
  connectMock.mockReset();
  connectMock.mockResolvedValue(conn);
  ({ handleContentMessage: handle } = await import("../src/background/relay-handler"));
});

describe("relay handler: senders", () => {
  it.each([
    ["another extension", tabSender({ id: "someone-else" })],
    ["no tab (not a content script)", tabSender({ tab: undefined })],
  ])("ignores %s", async (_name, sender) => {
    handle(request(), sender as never);
    await flush();
    expect(conn.sent).toEqual([]);
    expect(repliesSent()).toEqual([]);
  });

  it.each([undefined, "", "not-a-token", 5])(
    "ignores a message with document %j",
    async (document) => {
      handle(request(undefined, { document }), tabSender() as never);
      await flush();
      expect(conn.sent).toEqual([]);
    },
  );
});

describe("relay handler: web context", () => {
  it("takes origin and topOrigin from the sender, never from the payload", async () => {
    const framed = tabSender({ url: "https://widget.example/f", frameId: 4 });
    handle(
      request({ type: "choose" }, { web: { origin: "https://evil.example" } }),
      framed as never,
    );
    await flush();
    expect(conn.sent).toMatchObject([
      {
        id: "7.4.p1",
        web: { origin: "https://widget.example", topOrigin: "https://app.example.com" },
      },
    ]);
  });

  it("asks the top frame's content script when the tab URL is withheld", async () => {
    fakeBrowser.tabs.sendMessage.mockResolvedValueOnce("https://top.example" as never);
    const framed = tabSender({ url: "https://widget.example/f", frameId: 4, tab: { id: 7 } });
    handle(request(), framed as never);
    await flush();
    expect(fakeBrowser.tabs.sendMessage).toHaveBeenCalledWith(
      7,
      { kind: "websign-origin" },
      { frameId: 0 },
    );
    expect(conn.sent).toMatchObject([{ web: { topOrigin: "https://top.example" } }]);
  });

  it.each(["http://app.example.com/", "file:///tmp/x.html", "data:text/html,x"])(
    "answers InsecureOrigin for %s without contacting the app",
    async (url) => {
      handle(request(), tabSender({ url, tab: { id: 7, url } }) as never);
      await flush();
      expect(connectMock).not.toHaveBeenCalled();
      expect(repliesSent()).toEqual([
        [
          7,
          {
            kind: "websign-reply",
            document: DOC,
            id: "p1",
            message: { type: "error", code: "InsecureOrigin", message: expect.any(String) },
          },
          { frameId: 0 },
        ],
      ]);
    },
  );

  it("answers InvalidRequest for a payload that fails validation", async () => {
    handle(request({ type: "sign.begin", hash: "MD5" }), tabSender() as never);
    await flush();
    expect(conn.sent).toEqual([]);
    expect(repliesSent()[0]?.[1]).toMatchObject({ message: { code: "InvalidRequest" } });
  });
});

describe("relay handler: replies and goodbyes", () => {
  it("sends app messages to the requesting tab and frame, tagged with its document", async () => {
    handle(request(), tabSender({ frameId: 2 }) as never);
    await flush();
    conn.emit("7.2.p1", { type: "choose.result", certificates: [] });
    expect(repliesSent()).toEqual([
      [
        7,
        {
          kind: "websign-reply",
          document: DOC,
          id: "p1",
          message: { type: "choose.result", certificates: [] },
        },
        { frameId: 2 },
      ],
    ]);
  });

  it("cancels the requests of a document that went away", async () => {
    handle(request(), tabSender() as never);
    await flush();
    handle({ kind: "websign-gone", document: DOC }, tabSender() as never);
    expect(conn.sentOfType("cancel").map((m) => m.id)).toEqual(["7.0.p1"]);
  });

  it("does not let a document cancel another tab's requests", async () => {
    handle(request(), tabSender() as never);
    await flush();
    handle(
      { kind: "websign-gone", document: DOC },
      tabSender({ tab: { id: 8, url: PAGE } }) as never,
    );
    expect(conn.sentOfType("cancel")).toEqual([]);
  });
});
