import { describe, expect, it, vi } from "vitest";
import { CHROME_WEB_STORE_ID, EDGE_ADDONS_ID, HOMEPAGE } from "../src/project";
import { loadSdk, useFakeEnvironment } from "./helpers/env";
import { flush } from "./helpers/fake-script";

useFakeEnvironment();

const AMO = "https://addons.mozilla.org/firefox/addon/websign/";
const DOWNLOAD = `${HOMEPAGE}download`;
// While a store ID in project.toml is empty, that store falls back to the download page (SPEC §7).
const CHROME = CHROME_WEB_STORE_ID
  ? `https://chromewebstore.google.com/detail/${CHROME_WEB_STORE_ID}`
  : DOWNLOAD;
const EDGE = EDGE_ADDONS_ID
  ? `https://microsoftedge.microsoft.com/addons/detail/${EDGE_ADDONS_ID}`
  : DOWNLOAD;

describe("installUrl()", () => {
  it("Firefox announced -> AMO", async () => {
    const env = await loadSdk({ browser: "firefox" });
    env.script.announce();
    expect(env.sdk.installUrl()).toBe(AMO);
  });

  for (const browser of ["chrome", "brave", "opera", "vivaldi", "chromium"]) {
    it(`${browser} announced -> Chrome Web Store (download page while unlisted)`, async () => {
      const env = await loadSdk({ browser });
      env.script.announce();
      expect(env.sdk.installUrl()).toBe(CHROME);
    });
  }

  it("edge announced -> Edge Add-ons (download page while unlisted)", async () => {
    const env = await loadSdk({ browser: "edge" });
    env.script.announce();
    expect(env.sdk.installUrl()).toBe(EDGE);
  });

  for (const browser of ["safari", "other"]) {
    it(`${browser} announced -> the download page`, async () => {
      const env = await loadSdk({ browser });
      env.script.announce();
      expect(env.sdk.installUrl()).toBe(DOWNLOAD);
    });
  }

  it("the announcement wins over the user agent", async () => {
    const env = await loadSdk({
      browser: "firefox",
      userAgent: "Mozilla/5.0 Chrome/129.0 Safari/537.36",
    });
    env.script.announce();
    expect(env.sdk.installUrl()).toBe(AMO);
  });

  it("without an announcement, detects Firefox from the user agent", async () => {
    const env = await loadSdk({
      userAgent: "Mozilla/5.0 (X11; Linux x86_64; rv:130.0) Gecko/20100101 Firefox/130.0",
    });
    expect(env.sdk.installUrl()).toBe(AMO);
  });

  it("without an announcement, detects Edge from the user agent", async () => {
    const env = await loadSdk({ userAgent: "Mozilla/5.0 Chrome/129.0 Safari/537.36 Edg/129.0" });
    expect(env.sdk.installUrl()).toBe(EDGE);
  });

  it("without an announcement, detects Chrome from the user agent", async () => {
    const env = await loadSdk({ userAgent: "Mozilla/5.0 Chrome/129.0 Safari/537.36" });
    expect(env.sdk.installUrl()).toBe(CHROME);
  });

  it("Safari user agent -> the download page", async () => {
    const env = await loadSdk({
      userAgent: "Mozilla/5.0 (Macintosh) AppleWebKit/605.1.15 Version/17.0 Safari/605.1.15",
    });
    expect(env.sdk.installUrl()).toBe(DOWNLOAD);
  });

  it("uses navigator.userAgentData brands when present", async () => {
    const env = await loadSdk();
    vi.stubGlobal("navigator", {
      userAgent: "",
      userAgentData: { brands: [{ brand: "Microsoft Edge", version: "129" }] },
    });
    expect(env.sdk.installUrl()).toBe(EDGE);
  });

  it("falls back to the download page without navigator, and never throws", async () => {
    const env = await loadSdk();
    vi.stubGlobal("navigator", undefined);
    expect(env.sdk.installUrl()).toBe(DOWNLOAD);
  });

  it("is synchronous and needs no extension", async () => {
    const env = await loadSdk({ answerDiscover: false });
    expect(typeof env.sdk.installUrl()).toBe("string");
    await flush();
    expect(env.win.posted).toEqual([]);
  });
});
