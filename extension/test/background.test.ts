import { beforeEach, describe, expect, it, vi } from "vitest";
import { fakeBrowser } from "./fakes/browser";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));
vi.mock("../src/background/browser", () => ({
  detectBrowser: async () => ({ name: "chrome", version: "121.0" }),
}));

beforeEach(async () => {
  fakeBrowser.reset();
  vi.resetModules();
  const background = (await import("../src/entrypoints/background")).default;
  background.main();
});

describe("background: introducing the extension to the app", () => {
  it.each(["install", "update"])("connects with reason installed on %s", async (reason) => {
    fakeBrowser.runtime.onInstalled.emit({ reason });
    await vi.waitFor(() => expect(fakeBrowser.ports).toHaveLength(1));
    await vi.waitFor(() =>
      expect(fakeBrowser.ports[0]?.sent[0]).toMatchObject({ browser: { reason: "installed" } }),
    );
  });

  it("does not start the app when only the browser was updated", () => {
    fakeBrowser.runtime.onInstalled.emit({ reason: "chrome_update" });
    expect(fakeBrowser.runtime.connectNative).not.toHaveBeenCalled();
  });

  it("does not start the app on every browser startup", () => {
    expect(fakeBrowser.runtime.onStartup.listeners.size).toBe(0);
  });
});
