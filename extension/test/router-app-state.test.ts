import { beforeEach, describe, expect, it, vi } from "vitest";
import type { PageReply, PageRequest } from "../src/generated";
import { MIN_APP_VERSION } from "../src/generated/project";
import { fakeBrowser } from "./fakes/browser";
import { FakeConnection } from "./fakes/connection";
import { helloReply } from "./fakes/protocol";

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
let AppError: typeof import("../src/background/app-error").AppError;
let replies: PageReply[];

async function send(request: PageRequest, pageId = "p1") {
  router.route({ tabId: 1, frameId: 0, context }, pageId, request, (m) => replies.push(m));
  await flush();
}

beforeEach(async () => {
  fakeBrowser.reset();
  vi.resetModules();
  connectMock.mockReset();
  replies = [];
  router = await import("../src/background/router");
  // After resetModules the router has a fresh app-error module; `instanceof` needs the same one.
  ({ AppError } = await import("../src/background/app-error"));
});

describe("app older than MIN_APP_VERSION (from project.toml)", () => {
  const old = "0.0.1";

  beforeEach(() => {
    connectMock.mockResolvedValue(new FakeConnection(helloReply(old)));
  });

  it("answers choose and sign.begin with AppOutdated and details, sending nothing", async () => {
    const conn = new FakeConnection(helloReply(old));
    connectMock.mockResolvedValue(conn);
    await send({ type: "choose" }, "p1");
    await send({ type: "sign.begin", hash: "SHA-256" }, "p2");
    expect(conn.sent).toEqual([]);
    expect(replies).toHaveLength(2);
    for (const r of replies) {
      expect(r).toMatchObject({
        type: "error",
        code: "AppOutdated",
        details: { installed: old, required: MIN_APP_VERSION },
      });
    }
  });

  it("answers status itself (appOutdated, the app found, not remembered) without asking the app", async () => {
    const conn = new FakeConnection(helloReply(old));
    connectMock.mockResolvedValue(conn);
    await send({ type: "status" });
    expect(conn.sent).toEqual([]);
    expect(replies).toHaveLength(1);
    expect(replies[0]).toMatchObject({
      type: "status",
      appOutdated: true,
      remembered: false,
      app: { version: old },
    });
  });

  it("does not flag an app exactly at the minimum", async () => {
    const conn = new FakeConnection(helloReply(MIN_APP_VERSION));
    connectMock.mockResolvedValue(conn);
    await send({ type: "choose" });
    expect(conn.sent).toHaveLength(1);
    expect(replies).toEqual([]);
  });
});

describe("native host missing", () => {
  it("answers requests with AppMissing", async () => {
    connectMock.mockRejectedValue(new AppError("AppMissing", "host not found"));
    await send({ type: "choose" });
    expect(replies).toMatchObject([{ type: "error", code: "AppMissing" }]);
  });

  it("answers requests with Internal when the app never said hello", async () => {
    connectMock.mockRejectedValue(new AppError("Internal", "the app did not respond"));
    await send({ type: "sign.begin", hash: "SHA-256" });
    expect(replies).toMatchObject([{ type: "error", code: "Internal" }]);
  });

  it("still answers status, with no app and appOutdated false", async () => {
    connectMock.mockRejectedValue(new AppError("AppMissing", "host not found"));
    await send({ type: "status" });
    expect(replies).toHaveLength(1);
    expect(replies[0]).toMatchObject({ type: "status", appOutdated: false, remembered: false });
    expect(replies[0]).not.toHaveProperty("app");
  });

  it("carries the app's details (a hello refused with AppOutdated)", async () => {
    const details = { installed: "0.9.0", required: "1.0.0" };
    connectMock.mockRejectedValue(new AppError("AppOutdated", "too old", details));
    await send({ type: "choose" });
    expect(replies).toEqual([{ type: "error", code: "AppOutdated", message: "too old", details }]);
  });

  it("answers Internal when connect rejects with anything but an AppError", async () => {
    connectMock.mockRejectedValue(new Error("boom"));
    await send({ type: "choose" });
    expect(replies).toMatchObject([{ type: "error", code: "Internal" }]);
  });
});
