/** The background's answers to the popup, and the toolbar badge (docs/ux.md §9). */

import { beforeEach, describe, expect, it, vi } from "vitest";
import { fakeBrowser } from "./fakes/browser";
import { FakeConnection } from "./fakes/connection";
import { helloReply } from "./fakes/protocol";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));
const connectMock = vi.hoisted(() => vi.fn());
vi.mock("../src/background/connection", () => ({ connect: connectMock }));

let handle: typeof import("../src/background/popup-handler").handlePopupMessage;
let AppError: typeof import("../src/background/app-error").AppError;
const popup = () => ({
  id: fakeBrowser.runtime.id,
  url: fakeBrowser.runtime.getURL("/popup.html"),
});
const flush = () => new Promise((resolve) => setTimeout(resolve, 0));

async function probe(sender: unknown = popup()) {
  const respond = vi.fn();
  const handled = handle({ kind: "websign-popup", op: "probe" }, sender as never, respond);
  await flush();
  return { handled, respond };
}
const badge = () => fakeBrowser.action.setBadgeText.mock.calls.at(-1)?.[0].text;

beforeEach(async () => {
  fakeBrowser.reset();
  vi.resetModules();
  connectMock.mockReset();
  ({ handlePopupMessage: handle } = await import("../src/background/popup-handler"));
  ({ AppError } = await import("../src/background/app-error"));
});

describe("popup handler", () => {
  it("reports the app version and clears the badge when ready", async () => {
    connectMock.mockResolvedValue(new FakeConnection(helloReply("9.0.0")));
    const { handled, respond } = await probe();
    expect(handled).toBe(true);
    expect(connectMock).toHaveBeenCalledWith("popup");
    expect(respond).toHaveBeenCalledWith({ ok: true, appVersion: "9.0.0" });
    expect(badge()).toBe("");
  });

  it.each([
    ["unreachable (plain Error)", () => new Error("x"), { ok: false, code: "Internal" }],
    ["missing", () => new AppError("AppMissing", "x"), { ok: false, code: "AppMissing" }],
  ])("shows the ! badge when the app is %s", async (_name, failure, expected) => {
    connectMock.mockRejectedValue(failure());
    const { respond } = await probe();
    expect(respond).toHaveBeenCalledWith(expected);
    expect(badge()).toBe("!");
  });

  it("carries the versions of an AppOutdated refusal", async () => {
    const details = { installed: "0.9.0", required: "1.0.0" };
    connectMock.mockRejectedValue(new AppError("AppOutdated", "old", details));
    const { respond } = await probe();
    expect(respond).toHaveBeenCalledWith({ ok: false, code: "AppOutdated", details });
    expect(badge()).toBe("!");
  });

  it("shows the ! badge for an app older than MIN_APP_VERSION", async () => {
    connectMock.mockResolvedValue(new FakeConnection(helloReply("0.0.1")));
    await probe();
    expect(badge()).toBe("!");
  });

  it.each([
    ["a content script (has a tab)", { ...popup(), tab: { id: 1 } }],
    ["another extension", { ...popup(), id: "other" }],
    ["a web page URL", { id: fakeBrowser.runtime.id, url: "https://evil.example/" }],
  ])("ignores %s", async (_name, sender) => {
    const { handled, respond } = await probe(sender);
    expect(handled).toBe(false);
    expect(respond).not.toHaveBeenCalled();
    expect(connectMock).not.toHaveBeenCalled();
  });

  it("opens diagnostics through the app", async () => {
    const conn = new FakeConnection(helloReply("9.0.0"));
    connectMock.mockResolvedValue(conn);
    const respond = vi.fn();
    handle({ kind: "websign-popup", op: "diagnostics" }, popup() as never, respond);
    await flush();
    const [sent] = conn.sentOfType("diagnostics.open");
    expect(sent?.id).toMatch(/^ext\.\d+$/);
    conn.emit(sent?.id ?? "", { type: "done" });
    await flush();
    expect(respond).toHaveBeenCalledWith({ ok: true, appVersion: "" });
  });
});
