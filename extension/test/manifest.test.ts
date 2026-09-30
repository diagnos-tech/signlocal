/** The manifest per target and channel: permissions, pinned ID, CSP, Gecko settings. */

import { describe, expect, it, vi } from "vitest";
import { channelFromEnv, EXTENSION_PAGES_CSP, manifestFor } from "../build/manifest";
import content from "../src/entrypoints/content";
import { DEV_KEY, FIREFOX_ID } from "../src/generated/project";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));

const CHROMIUM = ["chrome", "edge"];
const ALL = [...CHROMIUM, "firefox", "safari"];

describe("manifestFor", () => {
  it.each(ALL)("%s asks for nativeMessaging only and no host permissions", (browser) => {
    for (const channel of ["direct", "store"] as const) {
      const manifest = manifestFor({ browser, mode: "production", channel });
      expect(manifest.permissions).toEqual(["nativeMessaging"]);
      expect(manifest).not.toHaveProperty("host_permissions");
      expect(manifest).not.toHaveProperty("optional_permissions");
      expect(manifest).not.toHaveProperty("externally_connectable");
      expect(manifest).not.toHaveProperty("web_accessible_resources");
    }
  });

  it.each(CHROMIUM)(
    "%s direct (release zip, unpacked) carries dev_key to pin dev_id",
    (browser) => {
      expect(manifestFor({ browser, mode: "production", channel: "direct" }).key).toBe(DEV_KEY);
    },
  );

  it.each(CHROMIUM)("%s store builds never carry the key", (browser) => {
    expect(manifestFor({ browser, mode: "production", channel: "store" })).not.toHaveProperty(
      "key",
    );
  });

  it.each(CHROMIUM)("%s dev server builds carry the key whatever the channel", (browser) => {
    expect(manifestFor({ browser, mode: "development", channel: "store" }).key).toBe(DEV_KEY);
  });

  it.each(["firefox", "safari"])("%s never carries a Chromium key", (browser) => {
    expect(manifestFor({ browser, mode: "production", channel: "direct" })).not.toHaveProperty(
      "key",
    );
  });

  it("firefox pins the Gecko ID and minimum version", () => {
    expect(
      manifestFor({ browser: "firefox", mode: "production", channel: "direct" }),
    ).toMatchObject({
      browser_specific_settings: { gecko: { id: FIREFOX_ID, strict_min_version: "121.0" } },
    });
  });

  it.each(ALL)("%s production builds lock extension pages to their own files", (browser) => {
    const manifest = manifestFor({ browser, mode: "production", channel: "direct" });
    expect(manifest.content_security_policy).toEqual({ extension_pages: EXTENSION_PAGES_CSP });
    expect(EXTENSION_PAGES_CSP).toContain("default-src 'none'");
    expect(EXTENSION_PAGES_CSP).toContain("script-src 'self';");
    expect(EXTENSION_PAGES_CSP).not.toMatch(/unsafe|https?:|\*/);
  });

  it("localizes name and description", () => {
    expect(manifestFor({ browser: "chrome", mode: "production", channel: "direct" })).toMatchObject(
      {
        name: "__MSG_extension_name__",
        description: "__MSG_extension_description__",
        default_locale: "en",
      },
    );
  });
});

describe("content script registration (SPEC §1)", () => {
  it("matches secure contexts only, in all frames, at document_start", () => {
    expect([...(content.matches as string[])].sort()).toEqual(
      ["http://127.0.0.1/*", "http://[::1]/*", "http://localhost/*", "https://*/*"].sort(),
    );
    expect(content.allFrames).toBe(true);
    expect(content.runAt).toBe("document_start");
  });
});

describe("channelFromEnv", () => {
  it.each([
    [undefined, "direct"],
    ["", "direct"],
    ["direct", "direct"],
    ["store", "store"],
  ])("%j → %s", (value, channel) => {
    expect(channelFromEnv(value)).toBe(channel);
  });

  it("fails loudly on a typo instead of shipping the wrong ID", () => {
    expect(() => channelFromEnv("stroe")).toThrow(/WEBSIGN_CHANNEL/);
  });
});
