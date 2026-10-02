// @vitest-environment happy-dom
/** The popup's accessibility, focus, links and Open diagnostics (docs/ux.md §9, §14). */

import { beforeEach, describe, expect, it, vi } from "vitest";
import { fakeBrowser } from "./fakes/browser";
import { createTab, HOME, primary, secondary, setUpPopup, shown, title } from "./popup-harness";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));

beforeEach(setUpPopup);

const status = () => document.querySelector(".status");

describe("popup a11y: announcements", () => {
  it("keeps one status region and changes only its content", async () => {
    let answer: (result: { ok: true; appVersion: string }) => void = () => {};
    await shown(new Promise((resolve) => (answer = resolve)), "linux", 0);
    const region = status();
    expect(region?.getAttribute("role")).toBe("status");
    expect(region?.children).toHaveLength(0);
    await vi.advanceTimersByTimeAsync(150);
    expect(region?.getAttribute("aria-busy")).toBe("true");
    answer({ ok: true, appVersion: "1.4.0" });
    await vi.advanceTimersByTimeAsync(0);
    expect(status()).toBe(region);
    expect(region?.getAttribute("aria-busy")).toBe("false");
    expect(document.querySelectorAll("[role=status], [aria-live]")).toHaveLength(1);
  });

  it("sets the page language and title", async () => {
    await shown({ ok: true, appVersion: "1.4.0" });
    expect(document.documentElement.lang).toBe("en-US");
    expect(document.title).toBe("extension_name");
    expect(document.querySelector("h1")?.textContent).toBe("popup_ready_title");
  });

  it("hides every icon from assistive technology", async () => {
    await shown({ ok: false, code: "Timeout" });
    const icons = [...document.querySelectorAll("svg")];
    expect(icons.length).toBeGreaterThan(2);
    for (const svg of icons) {
      expect(svg.getAttribute("aria-hidden")).toBe("true");
      expect(svg.getAttribute("focusable")).toBe("false");
    }
  });

  it("gives buttons an explicit type and a visible label", async () => {
    await shown({ ok: true, appVersion: "1.4.0" });
    await shown({ ok: false, code: "Timeout" });
    for (const button of document.querySelectorAll("button")) {
      expect(button.type).toBe("button");
      expect(button.textContent?.trim()).not.toBe("");
    }
  });
});

describe("popup a11y: links", () => {
  it("says every link opens a new tab and never hands over the opener", async () => {
    await shown({ ok: false, code: "AppMissing" });
    const hint = document.getElementById("new-tab-hint");
    expect(hint?.textContent).toBe("popup_new_tab");
    expect(hint?.hidden).toBe(true);
    const links = [...document.querySelectorAll("a")];
    expect(links).toHaveLength(3);
    for (const link of links) {
      expect(link.getAttribute("aria-describedby")).toBe("new-tab-hint");
      expect(link.target).toBe("_blank");
      expect(link.rel).toBe("noopener noreferrer");
      expect(link.href.startsWith(HOME)).toBe(true);
    }
  });

  it("opens a plain click through tabs.create, then closes the popup", async () => {
    const close = vi.spyOn(window, "close").mockImplementation(() => {});
    await shown({ ok: false, code: "AppMissing" }, "win");
    const click = new MouseEvent("click", { bubbles: true, cancelable: true, button: 0 });
    primary()?.dispatchEvent(click);
    await vi.advanceTimersByTimeAsync(0);
    expect(click.defaultPrevented).toBe(true);
    expect(createTab).toHaveBeenCalledWith({ url: `${HOME}download.html#windows` });
    expect(close).toHaveBeenCalledTimes(1);
  });

  it("leaves modified clicks to the browser (background tab, new window)", async () => {
    await shown({ ok: false, code: "AppMissing" });
    const click = new MouseEvent("click", { bubbles: true, cancelable: true, ctrlKey: true });
    primary()?.dispatchEvent(click);
    expect(click.defaultPrevented).toBe(false);
    expect(createTab).not.toHaveBeenCalled();
  });
});

describe("popup a11y: focus", () => {
  it("does not steal focus when the popup opens", async () => {
    await shown({ ok: false, code: "AppMissing" });
    expect(document.activeElement).toBe(document.body);
  });

  it("moves focus from Try again to the next state's primary action", async () => {
    await shown({ ok: false, code: "Timeout" });
    primary()?.focus();
    fakeBrowser.runtime.sendMessage.mockResolvedValue({ ok: false, code: "AppMissing" });
    primary()?.click();
    await vi.advanceTimersByTimeAsync(0);
    expect(title()).toBe("popup_missing_title");
    expect(document.activeElement).toBe(primary());
  });
});

describe("popup: Open diagnostics", () => {
  it("asks the background once and closes the popup", async () => {
    const close = vi.spyOn(window, "close").mockImplementation(() => {});
    await shown({ ok: true, appVersion: "1.4.0" });
    let opened: (result: { ok: true; appVersion: string }) => void = () => {};
    fakeBrowser.runtime.sendMessage.mockClear();
    fakeBrowser.runtime.sendMessage.mockImplementation(
      () => new Promise((resolve) => (opened = resolve)),
    );
    secondary()?.click();
    secondary()?.click();
    expect(document.querySelector(".actions")?.getAttribute("aria-busy")).toBe("true");
    expect(fakeBrowser.runtime.sendMessage).toHaveBeenCalledTimes(1);
    expect(fakeBrowser.runtime.sendMessage).toHaveBeenCalledWith({
      kind: "websign-popup",
      op: "diagnostics",
    });
    opened({ ok: true, appVersion: "" });
    await vi.advanceTimersByTimeAsync(0);
    expect(close).toHaveBeenCalledTimes(1);
  });

  it("stays open and re-checks when the app could not open them", async () => {
    const close = vi.spyOn(window, "close").mockImplementation(() => {});
    await shown({ ok: true, appVersion: "1.4.0" });
    fakeBrowser.runtime.sendMessage.mockImplementation(async (message: unknown) =>
      (message as { op?: string }).op === "probe"
        ? { ok: false, code: "AppMissing" }
        : { ok: false, code: "Internal" },
    );
    secondary()?.click();
    await vi.advanceTimersByTimeAsync(0);
    expect(close).not.toHaveBeenCalled();
    expect(title()).toBe("popup_missing_title");
    expect(document.querySelector(".actions")?.hasAttribute("aria-busy")).toBe(false);
  });
});
