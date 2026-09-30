import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { PageReply } from "../src/generated";
import { FakeAppex } from "./fakes/appex";
import { fakeBrowser } from "./fakes/browser";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));
vi.mock("../src/background/browser", () => ({
  detectBrowser: async () => ({ name: "safari", version: "18.0" }),
}));

type Router = typeof import("../src/background/router");
type Requests = typeof import("../src/background/requests");
let router: Router;
let requests: Requests;
let appex: FakeAppex;
const settle = () => vi.advanceTimersByTimeAsync(0);
const sender = (tabId: number) => ({
  tabId,
  frameId: 0,
  context: { origin: `https://tab${tabId}.example`, topOrigin: `https://tab${tabId}.example` },
});

beforeEach(async () => {
  vi.useFakeTimers();
  fakeBrowser.reset();
  fakeBrowser.urlScheme = "safari-web-extension";
  appex = new FakeAppex();
  fakeBrowser.runtime.sendNativeMessage.mockImplementation(appex.handle);
  vi.resetModules();
  router = await import("../src/background/router");
  requests = await import("../src/background/requests");
});

afterEach(() => {
  vi.useRealTimers();
});

describe("Safari transport: two tabs on the shared session", () => {
  it("delivers each reply only to the tab that asked", async () => {
    const five: PageReply[] = [];
    const seven: PageReply[] = [];
    router.route(sender(5), "p1", { type: "choose" }, (m) => five.push(m));
    router.route(sender(7), "p1", { type: "choose" }, (m) => seven.push(m));
    await settle();
    expect(appex.sessions.size).toBe(1);
    const received = appex.session.received.map((m) => [m.type, m.id, m.web]);
    expect(received).toContainEqual(["choose", "5.0.p1", sender(5).context]);
    expect(received).toContainEqual(["choose", "7.0.p1", sender(7).context]);

    const certificate = (tab: number) => [{ certificate: `cert-of-${tab}` }];
    appex.say(appex.session, {
      v: 1,
      id: "7.0.p1",
      type: "choose.result",
      certificates: certificate(7),
    });
    await settle();
    appex.say(appex.session, {
      v: 1,
      id: "5.0.p1",
      type: "choose.result",
      certificates: certificate(5),
    });
    await settle();
    expect(five).toEqual([{ type: "choose.result", certificates: certificate(5) }]);
    expect(seven).toEqual([{ type: "choose.result", certificates: certificate(7) }]);
  });

  it("closing one tab cancels only its request, through the relay", async () => {
    const seven: PageReply[] = [];
    router.route(sender(5), "p1", { type: "sign.begin", hash: "SHA-256" }, () => {});
    router.route(sender(7), "p1", { type: "sign.begin", hash: "SHA-256" }, (m) => seven.push(m));
    await settle();
    requests.cancelTab(5);
    await settle();
    const cancels = appex.session.received.filter((m) => m.type === "cancel").map((m) => m.id);
    expect(cancels).toEqual(["5.0.p1"]);
    appex.say(appex.session, { v: 1, id: "5.0.p1", type: "error", code: "Aborted", message: "x" });
    appex.say(appex.session, {
      v: 1,
      id: "7.0.p1",
      type: "sign.need_digest",
      seq: 1,
      certificate: {},
    });
    await settle();
    expect(seven).toHaveLength(1);
    expect(seven[0]).toMatchObject({ type: "sign.need_digest" });
  });

  it("the host exiting fails every open request with Internal", async () => {
    const replies: PageReply[] = [];
    router.route(sender(5), "p1", { type: "choose" }, (m) => replies.push(m));
    router.route(sender(7), "p1", { type: "choose" }, (m) => replies.push(m));
    await settle();
    appex.exit(appex.session);
    await settle();
    expect(replies).toHaveLength(2);
    for (const reply of replies) expect(reply).toMatchObject({ type: "error", code: "Internal" });
  });
});
