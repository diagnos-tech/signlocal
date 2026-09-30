// @vitest-environment happy-dom
/** The popup's DOM per state (docs/ux.md §9): texts, links, actions and accessibility. */

import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { ProbeResult } from "../src/shared/runtime-messages";
import { fakeBrowser } from "./fakes/browser";

vi.mock("wxt/browser", async () => ({ browser: (await import("./fakes/browser")).fakeBrowser }));

const HOME = "https://diagnos-tech.github.io/web-esign/";
let root: HTMLElement;
let render: typeof import("../src/popup/view").render;

/** getMessage that shows its substitutions, so the test sees what was filled in. */
function messages(key: string, substitutions?: string | string[]): string {
  const list = substitutions === undefined ? [] : [substitutions].flat();
  return list.length > 0 ? `${key}(${list.join(",")})` : key;
}

function answerProbe(result: ProbeResult | Promise<ProbeResult>): void {
  fakeBrowser.runtime.sendMessage.mockImplementation(async (message: unknown) =>
    (message as { op?: string }).op === "probe" ? result : undefined,
  );
}

async function shown(result: ProbeResult, os = "linux"): Promise<void> {
  fakeBrowser.runtime.getPlatformInfo.mockResolvedValue({ os });
  answerProbe(result);
  render(root);
  await vi.advanceTimersByTimeAsync(0);
}

const title = () => root.querySelector("h1")?.textContent;
const primary = () => root.querySelector<HTMLElement>(".actions .btn");
const secondary = () => root.querySelector<HTMLAnchorElement>(".actions .link");

beforeEach(async () => {
  vi.useFakeTimers();
  fakeBrowser.reset();
  fakeBrowser.manifestVersion = "1.4.2";
  fakeBrowser.i18n.getMessage.mockImplementation(messages);
  document.body.innerHTML = '<main id="app" aria-live="polite"></main>';
  root = document.getElementById("app") as HTMLElement;
  ({ render } = await import("../src/popup/view"));
});

afterEach(() => {
  vi.useRealTimers();
});

describe("popup view: states", () => {
  it("ready: title, version, and an Open diagnostics button with focus", async () => {
    await shown({ ok: true, appVersion: "1.4.0" });
    expect(title()).toBe("popup_ready_title");
    expect(root.querySelector(".card p")?.textContent).toBe("popup_ready_body(1.4.0)");
    expect(primary()?.tagName).toBe("BUTTON");
    expect(primary()?.textContent).toBe("popup_open_diagnostics");
    expect(document.activeElement).toBe(primary());
    expect(secondary()).toBeNull();
  });

  it("missing: download for the OS and the activation page", async () => {
    await shown({ ok: false, code: "AppMissing" }, "win");
    expect(title()).toBe("popup_missing_title");
    const download = root.querySelector<HTMLAnchorElement>(".actions a.btn");
    expect(download?.textContent).toBe("popup_download(Windows)");
    expect(download?.href).toBe(`${HOME}download.html`);
    expect(secondary()?.textContent).toBe("popup_activate");
    expect(secondary()?.href).toBe(`${HOME}activate/`);
  });

  it("outdated: both versions and the update link", async () => {
    await shown({ ok: true, appVersion: "0.0.1" }, "mac");
    expect(title()).toBe("popup_outdated_title");
    expect(root.querySelector(".card p")?.textContent).toMatch(/^popup_outdated_body\(0\.0\.1,/);
    expect(primary()?.textContent).toBe("popup_update_in(store_apple)");
    expect((primary() as HTMLAnchorElement).href).toBe(`${HOME}download.html`);
  });

  it("outdated from an AppOutdated refusal shows the versions the app sent", async () => {
    await shown({ ok: false, code: "AppOutdated", details: { installed: "0.9", required: "1.0" } });
    expect(root.querySelector(".card p")?.textContent).toBe("popup_outdated_body(0.9,1.0)");
    expect(root.querySelector("footer span")?.textContent).toBe("popup_footer_versions(1.4.2,0.9)");
  });

  it("error: the code in monospace, Try again probes again", async () => {
    await shown({ ok: false, code: "Internal" });
    expect(title()).toBe("popup_error_title");
    expect(root.querySelector(".card .mono")?.textContent).toBe("Internal");
    expect(secondary()?.href).toBe(`${HOME}download.html`);
    answerProbe({ ok: true, appVersion: "1.4.0" });
    primary()?.click();
    await vi.advanceTimersByTimeAsync(0);
    expect(title()).toBe("popup_ready_title");
  });

  it("unsupported: no probe, no actions", async () => {
    await shown({ ok: true, appVersion: "1.4.0" }, "android");
    expect(title()).toBe("popup_unsupported_title");
    expect(root.querySelector(".actions")?.children).toHaveLength(0);
    expect(fakeBrowser.runtime.sendMessage).not.toHaveBeenCalled();
  });

  it("checking: shown only when the probe takes longer than 150 ms", async () => {
    let answer: (result: ProbeResult) => void = () => {};
    fakeBrowser.runtime.getPlatformInfo.mockResolvedValue({ os: "linux" });
    answerProbe(new Promise<ProbeResult>((resolve) => (answer = resolve)));
    render(root);
    await vi.advanceTimersByTimeAsync(149);
    expect(title()).toBeUndefined();
    await vi.advanceTimersByTimeAsync(1);
    expect(title()).toBe("popup_checking");
    expect(root.querySelector(".i-spinner")).not.toBeNull();
    answer({ ok: true, appVersion: "1.4.0" });
    await vi.advanceTimersByTimeAsync(0);
    expect(title()).toBe("popup_ready_title");
  });
});

describe("popup view: structure and accessibility", () => {
  it("announces the card as a status inside a polite live region", async () => {
    await shown({ ok: true, appVersion: "1.4.0" });
    expect(root.getAttribute("aria-live")).toBe("polite");
    expect(root.querySelector(".card")?.getAttribute("role")).toBe("status");
    expect(document.documentElement.lang).toBe("en-US");
    expect(document.title).toBe("extension_name");
  });

  it("hides every icon from assistive technology", async () => {
    await shown({ ok: false, code: "AppMissing" });
    const icons = [...root.querySelectorAll("svg")];
    expect(icons.length).toBeGreaterThan(0);
    for (const svg of icons) {
      expect(svg.getAttribute("aria-hidden")).toBe("true");
      expect(svg.getAttribute("focusable")).toBe("false");
    }
  });

  it("opens every link in a new tab without an opener", async () => {
    await shown({ ok: false, code: "AppMissing" });
    const links = [...root.querySelectorAll("a")];
    expect(links.map((a) => a.href)).toContain(`${HOME}privacy.html`);
    for (const link of links) {
      expect(link.target).toBe("_blank");
      expect(link.rel).toBe("noopener noreferrer");
      expect(link.href.startsWith(HOME)).toBe(true);
    }
  });

  it("gives buttons an explicit type and a visible label", async () => {
    await shown({ ok: false, code: "Timeout" });
    for (const button of root.querySelectorAll("button")) {
      expect(button.type).toBe("button");
      expect(button.textContent?.trim()).not.toBe("");
    }
  });

  it("shows the extension and app versions in the footer", async () => {
    await shown({ ok: true, appVersion: "1.4.0" });
    expect(root.querySelector("footer span")?.textContent).toBe(
      "popup_footer_versions(1.4.2,1.4.0)",
    );
  });
});

describe("popup view: Open diagnostics", () => {
  it("asks the background and closes the popup", async () => {
    const close = vi.spyOn(window, "close").mockImplementation(() => {});
    await shown({ ok: true, appVersion: "1.4.0" });
    primary()?.click();
    await vi.advanceTimersByTimeAsync(0);
    expect(fakeBrowser.runtime.sendMessage).toHaveBeenCalledWith({
      kind: "websign-popup",
      op: "diagnostics",
    });
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
    primary()?.click();
    await vi.advanceTimersByTimeAsync(0);
    expect(close).not.toHaveBeenCalled();
    expect(title()).toBe("popup_missing_title");
  });
});
