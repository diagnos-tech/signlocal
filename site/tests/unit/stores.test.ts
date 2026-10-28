import { describe, expect, test } from "bun:test";
import { extensionSource, type StoreIds } from "../../src/platform/stores.ts";

const empty: StoreIds = { chromeWebStoreId: "", edgeAddonsId: "", firefoxAmoSlug: "" };
const chromeOnly: StoreIds = { ...empty, chromeWebStoreId: "abcdefghijklmnopabcdefghijklmnop" };
const all: StoreIds = { ...chromeOnly, edgeAddonsId: "edge123", firefoxAmoSlug: "signlocal" };

describe("extensionSource", () => {
  test("with no store IDs every browser gets the developer install", () => {
    for (const browser of [
      "chrome",
      "edge",
      "firefox",
      "opera",
      "brave",
      "chromium",
      "safari",
    ] as const) {
      expect(extensionSource(browser, empty).channel).toBe("dev");
    }
  });

  test("a Chrome Web Store ID switches Chrome, Brave and other Chromium browsers to the store", () => {
    for (const browser of ["chrome", "brave", "chromium"] as const) {
      const source = extensionSource(browser, chromeOnly);
      expect(source.channel).toBe("store");
      expect(source.store).toEqual({
        name: "Chrome Web Store",
        url: "https://chromewebstore.google.com/detail/abcdefghijklmnopabcdefghijklmnop",
      });
    }
  });

  test("Edge uses Edge Add-ons when it has an ID, else the Chrome Web Store with a note", () => {
    expect(extensionSource("edge", all).store?.name).toBe("Edge Add-ons");
    expect(extensionSource("edge", all).store?.url).toBe(
      "https://microsoftedge.microsoft.com/addons/detail/edge123",
    );
    const viaChrome = extensionSource("edge", chromeOnly);
    expect(viaChrome.store?.name).toBe("Chrome Web Store");
    expect(viaChrome.edgeUsesChromeStore).toBe(true);
  });

  test("Opera uses the Chrome Web Store and needs its add-on first", () => {
    const source = extensionSource("opera", chromeOnly);
    expect(source.store?.name).toBe("Chrome Web Store");
    expect(source.operaNeedsAddon).toBe(true);
    expect(extensionSource("opera", empty).operaNeedsAddon).toBe(false);
  });

  test("Firefox links to its listing only once the slug exists", () => {
    expect(extensionSource("firefox", chromeOnly).channel).toBe("dev");
    expect(extensionSource("firefox", all).store).toEqual({
      name: "Firefox Add-ons",
      url: "https://addons.mozilla.org/firefox/addon/signlocal/",
    });
  });

  test("Safari never uses a store: the extension is inside the app", () => {
    expect(extensionSource("safari", all).store).toBeUndefined();
  });
});
