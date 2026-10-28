import { describe, expect, test } from "bun:test";
import type { Detection } from "../../src/platform/types.ts";
import { type Context, complete, initial, reduce } from "../../src/wizard/state.ts";
import type { DeviceKind } from "../../src/wizard/url.ts";

const kinds: Record<string, DeviceKind> = { token1: "token", card1: "card", reader1: "reader" };
const detecting = (partial: Partial<Detection>): Context => ({
  detection: { platform: null, arch: null, browser: null, distro: null, ...partial },
  deviceKind: (id) => kinds[id],
});
const windowsChrome = detecting({ platform: "windows", arch: "x64", browser: "chrome" });
const macSafari = detecting({ platform: "macos", browser: "safari" });
const nothing = detecting({});

describe("initial", () => {
  test("a fresh visit opens at System with the detected system, processor and browser preselected", () => {
    const state = initial({}, undefined, windowsChrome);
    expect(state.step).toBe("system");
    expect(state.choices).toEqual({ os: "windows", arch: "x64", browser: "chrome" });
    expect(state.provided.size).toBe(0);
  });

  test("a certificate is never preselected", () => {
    expect(initial({}, undefined, windowsChrome).choices.cert).toBeUndefined();
  });

  test("x64 is preselected, not detected, when the system is chosen and the processor unknown", () => {
    const state = initial({ os: "linux" }, undefined, nothing);
    expect(state.choices.arch).toBe("x64");
    expect(state.provided.has("arch")).toBe(false);
  });

  test("?os=windows opens at Browser", () => {
    const state = initial({ os: "windows" }, undefined, windowsChrome);
    expect(state.step).toBe("browser");
    expect(state.choices.browser).toBe("chrome");
    expect(state.provided.has("browser")).toBe(false);
  });

  test("the popup's #macos opens at Browser with the detected browser preselected", () => {
    const state = initial({ os: "macos" }, undefined, macSafari);
    expect(state.step).toBe("browser");
    expect(state.choices).toEqual({ os: "macos", browser: "safari" });
  });

  test("Linux needs a distribution before the system step is done", () => {
    expect(initial({ os: "linux" }, undefined, nothing).step).toBe("system");
    expect(initial({ os: "linux", distro: "deb" }, undefined, nothing).step).toBe("browser");
  });

  test("a full shared link opens at Your steps", () => {
    const full = { os: "macos", browser: "chrome", cert: "file" } as const;
    expect(initial(full, "result", nothing).step).toBe("result");
    expect(initial(full, undefined, nothing).step).toBe("result");
  });

  test("a step is clamped to the first incomplete one", () => {
    expect(initial({ os: "macos" }, "result", nothing).step).toBe("browser");
    expect(initial({}, "certificate", nothing).step).toBe("system");
  });

  test("a browser invalid for the system is not preselected", () => {
    const state = initial({ os: "windows" }, undefined, macSafari);
    expect(state.choices.browser).toBeUndefined();
  });

  test("a browser from the URL invalid for the detected system is dropped", () => {
    const state = initial({ browser: "safari" }, undefined, windowsChrome);
    expect(state.choices.browser).toBe("chrome");
    expect(state.provided.has("browser")).toBe(false);
  });

  test("phones, ChromeOS and unknown systems preselect no system", () => {
    for (const platform of ["mobile", "chromeos", null] as const) {
      expect(
        initial({}, undefined, detecting({ platform, browser: "chrome" })).choices.os,
      ).toBeUndefined();
    }
  });
});

describe("select", () => {
  test("choosing a system marks it provided and keeps compatible choices", () => {
    const start = initial({}, undefined, windowsChrome);
    const next = reduce(start, { type: "select", field: "os", value: "linux" }, windowsChrome);
    expect(next.choices).toEqual({ os: "linux", arch: "x64", browser: "chrome" });
    expect(next.provided.has("os")).toBe(true);
  });

  test("leaving the Mac removes Safari with a notice", () => {
    const start = reduce(
      initial({ os: "macos" }, undefined, macSafari),
      { type: "select", field: "browser", value: "safari" },
      macSafari,
    );
    const next = reduce(start, { type: "select", field: "os", value: "windows" }, macSafari);
    expect(next.choices.browser).toBeUndefined();
    expect(next.notice).toBe("browser-reset");
    expect(next.provided.has("browser")).toBe(false);
  });

  test("Next from System keeps the notice for the Browser step, and Next from there drops it", () => {
    // A Mac with Safari: the system step opens with both preselected.
    const start = reduce(
      initial({}, undefined, macSafari),
      { type: "select", field: "os", value: "windows" },
      macSafari,
    );
    expect(start.notice).toBe("browser-reset");
    const atBrowser = reduce(start, { type: "next" }, macSafari);
    expect(atBrowser.step).toBe("browser");
    expect(atBrowser.notice).toBe("browser-reset");
    const chosen = reduce(
      atBrowser,
      { type: "select", field: "browser", value: "chrome" },
      macSafari,
    );
    expect(chosen.notice).toBeUndefined();
    expect(reduce(chosen, { type: "next" }, macSafari).notice).toBeUndefined();
  });

  test("choosing a browser clears the notice", () => {
    const start = {
      ...initial({ os: "macos" }, undefined, macSafari),
      notice: "browser-reset" as const,
    };
    const next = reduce(start, { type: "select", field: "browser", value: "chrome" }, macSafari);
    expect(next.notice).toBeUndefined();
  });

  test("macOS drops the processor, a system other than Linux drops the distribution", () => {
    let state = initial({ os: "linux", distro: "deb", arch: "arm64" }, undefined, nothing);
    state = reduce(state, { type: "select", field: "os", value: "macos" }, nothing);
    expect(state.choices.arch).toBeUndefined();
    expect(state.choices.distro).toBeUndefined();
  });

  test("Windows gets the detected processor, else x64, when it had none", () => {
    const arm = detecting({ platform: "macos", arch: "arm64" });
    let state = initial({ os: "macos" }, undefined, arm);
    state = reduce(state, { type: "select", field: "os", value: "windows" }, arm);
    expect(state.choices.arch).toBe("arm64");
    state = reduce(
      initial({ os: "macos" }, undefined, nothing),
      { type: "select", field: "os", value: "linux" },
      nothing,
    );
    expect(state.choices.arch).toBe("x64");
  });

  test("a certificate choice drops a device of another kind", () => {
    let state = initial({ cert: "token", device: "token1" }, undefined, nothing);
    expect(state.choices.device).toBe("token1");
    state = reduce(state, { type: "select", field: "cert", value: "card" }, nothing);
    expect(state.choices.device).toBeUndefined();
    state = reduce(state, { type: "select", field: "cert", value: "unsure" }, nothing);
    state = reduce(state, { type: "select", field: "device", value: "card1" }, nothing);
    expect(state.choices.device).toBe("card1");
  });

  test("a device that does not exist or does not fit is ignored", () => {
    let state = initial({ cert: "token" }, undefined, nothing);
    state = reduce(state, { type: "select", field: "device", value: "nope" }, nothing);
    state = reduce(state, { type: "select", field: "device", value: "card1" }, nothing);
    state = reduce(state, { type: "select", field: "device", value: "reader1" }, nothing);
    expect(state.choices.device).toBeUndefined();
  });

  test("invalid values are ignored", () => {
    const start = initial({ os: "windows" }, undefined, nothing);
    expect(reduce(start, { type: "select", field: "browser", value: "safari" }, nothing)).toEqual(
      start,
    );
    expect(reduce(start, { type: "select", field: "arch", value: "mips" }, nothing)).toEqual(start);
    expect(reduce(start, { type: "select", field: "os", value: "beos" }, nothing)).toEqual(start);
  });

  test("selecting clears the error of its step", () => {
    let state = initial({}, undefined, nothing);
    state = reduce(state, { type: "next" }, nothing);
    expect(state.errors.system).toBe(true);
    state = reduce(state, { type: "select", field: "os", value: "macos" }, nothing);
    expect(state.errors.system).toBeUndefined();
  });
});

describe("next, back, goto, restart", () => {
  test("Next on an incomplete step flags the error and stays", () => {
    const state = reduce(initial({}, undefined, nothing), { type: "next" }, nothing);
    expect(state.step).toBe("system");
    expect(state.errors.system).toBe(true);
  });

  test("Next confirms the preselected fields of the step", () => {
    const start = initial({}, undefined, windowsChrome);
    const next = reduce(start, { type: "next" }, windowsChrome);
    expect(next.step).toBe("browser");
    expect(next.provided.has("os")).toBe(true);
    expect(next.provided.has("arch")).toBe(true);
    expect(next.provided.has("browser")).toBe(false);
  });

  test("Back goes to the previous step and does nothing on System", () => {
    const start = initial({ os: "macos", browser: "chrome" }, "certificate", nothing);
    expect(reduce(start, { type: "back" }, nothing).step).toBe("browser");
    const first = initial({}, undefined, nothing);
    expect(reduce(first, { type: "back" }, nothing)).toEqual(first);
  });

  test("goto works only when every step before is complete", () => {
    const start = initial({ os: "macos" }, undefined, nothing);
    expect(reduce(start, { type: "goto", step: "certificate" }, nothing).step).toBe("browser");
    const done = initial({ os: "macos", browser: "chrome", cert: "token" }, "result", nothing);
    expect(reduce(done, { type: "goto", step: "browser" }, nothing).step).toBe("browser");
  });

  test("a complete walk reaches Your steps", () => {
    let state = initial({}, undefined, windowsChrome);
    state = reduce(state, { type: "next" }, windowsChrome);
    state = reduce(state, { type: "next" }, windowsChrome);
    state = reduce(state, { type: "select", field: "cert", value: "token" }, windowsChrome);
    state = reduce(state, { type: "next" }, windowsChrome);
    expect(state.step).toBe("result");
    expect(state.choices).toEqual({ os: "windows", arch: "x64", browser: "chrome", cert: "token" });
  });

  test("restart goes back to a fresh System step with detection preselected", () => {
    const done = initial(
      { os: "linux", distro: "deb", browser: "firefox", cert: "card" },
      "result",
      windowsChrome,
    );
    const again = reduce(done, { type: "restart" }, windowsChrome);
    expect(again.step).toBe("system");
    expect(again.choices).toEqual({ os: "windows", arch: "x64", browser: "chrome" });
    expect(again.provided.size).toBe(0);
  });
});

describe("complete", () => {
  test("System needs the processor on Windows and Linux and the distribution on Linux", () => {
    expect(complete("system", { os: "windows" })).toBe(false);
    expect(complete("system", { os: "windows", arch: "arm64" })).toBe(true);
    expect(complete("system", { os: "macos" })).toBe(true);
    expect(complete("system", { os: "linux", arch: "x64" })).toBe(false);
    expect(complete("system", { os: "linux", arch: "x64", distro: "rpm" })).toBe(true);
  });

  test("Browser needs a browser that exists on the system", () => {
    expect(complete("browser", { os: "windows", browser: "safari" })).toBe(false);
    expect(complete("browser", { os: "macos", browser: "safari" })).toBe(true);
    expect(complete("browser", { browser: "chrome" })).toBe(false);
  });
});
