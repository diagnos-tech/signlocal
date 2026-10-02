// @vitest-environment happy-dom
/** The popup's DOM per state (docs/ux.md §9): texts, links and actions. */

import { beforeEach, describe, expect, it, vi } from "vitest";
import { fakeBrowser } from "./fakes/browser";
import { HOME, primary, secondary, setUpPopup, shown, title } from "./popup-harness";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));

beforeEach(setUpPopup);

describe("popup view: states", () => {
  it("ready: title, version, Test your setup, then Open diagnostics", async () => {
    await shown({ ok: true, appVersion: "1.4.0" });
    expect(title()).toBe("popup_ready_title");
    expect(document.querySelector(".status p")?.textContent).toBe("popup_ready_body(1.4.0)");
    expect(primary()?.textContent).toBe("popup_test");
    expect((primary() as HTMLAnchorElement).href).toBe(`${HOME}test/`);
    expect(secondary()?.tagName).toBe("BUTTON");
    expect(secondary()?.textContent).toBe("popup_open_diagnostics");
  });

  it("missing: download for the OS at its section, and the setup page", async () => {
    await shown({ ok: false, code: "AppMissing" }, "win");
    expect(title()).toBe("popup_missing_title");
    expect(primary()?.textContent).toBe("popup_download(Windows)");
    expect((primary() as HTMLAnchorElement).href).toBe(`${HOME}download.html#windows`);
    expect(secondary()?.textContent).toBe("popup_activate");
    expect((secondary() as HTMLAnchorElement).href).toBe(`${HOME}activate/`);
  });

  it("outdated: both versions and the update download for the OS", async () => {
    await shown({ ok: true, appVersion: "0.0.1" }, "mac");
    expect(title()).toBe("popup_outdated_title");
    expect(document.querySelector(".status p")?.textContent).toMatch(
      /^popup_outdated_body\(0\.0\.1,/,
    );
    expect(primary()?.textContent).toBe("popup_update");
    expect((primary() as HTMLAnchorElement).href).toBe(`${HOME}download.html#macos`);
    expect(secondary()).toBeNull();
  });

  it("outdated from an AppOutdated refusal shows the versions the app sent", async () => {
    await shown({ ok: false, code: "AppOutdated", details: { installed: "0.9", required: "1.0" } });
    expect(document.querySelector(".status p")?.textContent).toBe("popup_outdated_body(0.9,1.0)");
  });

  it("error: the code for support, Try again probes again", async () => {
    await shown({ ok: false, code: "Internal" });
    expect(title()).toBe("popup_error_title");
    expect(document.querySelector(".status .mono")?.textContent).toBe("popup_error_code(Internal)");
    expect((secondary() as HTMLAnchorElement).href).toBe(`${HOME}download.html#linux`);
    fakeBrowser.runtime.sendMessage.mockResolvedValue({ ok: true, appVersion: "1.4.0" });
    primary()?.click();
    await vi.advanceTimersByTimeAsync(0);
    expect(title()).toBe("popup_ready_title");
  });

  it("unsupported: no probe, no actions", async () => {
    await shown({ ok: true, appVersion: "1.4.0" }, "android");
    expect(title()).toBe("popup_unsupported_title");
    expect(document.querySelector(".actions")?.children).toHaveLength(0);
    expect(fakeBrowser.runtime.sendMessage).not.toHaveBeenCalled();
  });

  it("checking: the frame at once, the spinner only after 150 ms", async () => {
    let answer: (result: { ok: true; appVersion: string }) => void = () => {};
    await shown(new Promise((resolve) => (answer = resolve)), "linux", 149);
    expect(document.querySelector("header")?.textContent).toBe("extension_name");
    expect(document.querySelector("footer")).not.toBeNull();
    expect(title()).toBeUndefined();
    await vi.advanceTimersByTimeAsync(1);
    expect(title()).toBe("popup_checking");
    expect(document.querySelector(".status .i-spinner")).not.toBeNull();
    answer({ ok: true, appVersion: "1.4.0" });
    await vi.advanceTimersByTimeAsync(0);
    expect(title()).toBe("popup_ready_title");
  });
});

describe("popup view: footer", () => {
  it("names both versions, filled by placeholder name, not position", async () => {
    await shown({ ok: true, appVersion: "1.4.0" });
    // $1 is {app} and $2 is {ext}: the catalog numbers placeholders alphabetically.
    expect(document.querySelector("footer span")?.textContent).toBe(
      "popup_footer_versions(1.4.0,1.4.2)",
    );
  });

  it("names only the extension while the app's version is unknown", async () => {
    await shown({ ok: false, code: "AppMissing" });
    expect(document.querySelector("footer span")?.textContent).toBe(
      "popup_footer_extension(1.4.2)",
    );
    expect(document.querySelector<HTMLAnchorElement>("footer a")?.href).toBe(`${HOME}privacy.html`);
  });
});
