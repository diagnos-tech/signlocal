import { beforeEach, describe, expect, it, vi } from "vitest";
import { announce } from "../src/content/announce";
import { fakeBrowser } from "./fakes/browser";
import { type FakeWindow, installWindow } from "./fakes/window";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));

let win: FakeWindow;

beforeEach(() => {
  fakeBrowser.reset();
  fakeBrowser.manifestVersion = "1.4.2";
  win = installWindow("https://app.example.com");
});

describe("announce", () => {
  it("posts the extension version and protocol range to the page's own origin", () => {
    announce();
    expect(win.posted).toHaveLength(1);
    const [posted] = win.posted;
    expect(posted?.targetOrigin).toBe("https://app.example.com");
    expect(posted?.message).toMatchObject({
      source: "websign-extension",
      kind: "announce",
      extension: { version: "1.4.2" },
      protocols: { min: 1, max: 1 },
    });
  });

  it("names the browser with a known BrowserName", () => {
    announce();
    const extension = win.posted[0]?.message.extension as { browser: string };
    expect([
      "chrome",
      "chromium",
      "edge",
      "brave",
      "opera",
      "vivaldi",
      "firefox",
      "safari",
      "other",
    ]).toContain(extension.browser);
  });

  it("announces again on every call (start, load and each discover)", () => {
    announce();
    announce();
    expect(win.posted).toHaveLength(2);
  });
});
