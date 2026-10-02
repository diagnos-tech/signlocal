import { describe, expect, it } from "vitest";
import { type PopupFacts, popupState } from "../src/popup/state";

const facts = (over: Partial<PopupFacts> = {}): PopupFacts => ({
  platform: "linux",
  probe: { ok: true, appVersion: "1.4.0" },
  minAppVersion: "1.3.0",
  ...over,
});

describe("popupState (docs/ux.md §9)", () => {
  it("checking while the probe runs", () => {
    expect(popupState(facts({ probe: null }))).toEqual({ kind: "checking" });
  });

  it("ready when the app is at or above the minimum", () => {
    expect(popupState(facts())).toEqual({ kind: "ready", appVersion: "1.4.0" });
    expect(popupState(facts({ probe: { ok: true, appVersion: "1.3.0" } })).kind).toBe("ready");
  });

  it("outdated when the app answers with an older version, carrying both versions", () => {
    expect(popupState(facts({ probe: { ok: true, appVersion: "1.1.0" } }))).toEqual({
      kind: "outdated",
      installed: "1.1.0",
      required: "1.3.0",
    });
  });

  it("outdated for a non-numeric app version (isOlder rule)", () => {
    expect(popupState(facts({ probe: { ok: true, appVersion: "1.4.0-beta" } })).kind).toBe(
      "outdated",
    );
  });

  it.each([
    ["linux", "linux"],
    ["mac", "macos"],
    ["win", "windows"],
  ] as const)("missing on %s keeps the OS for the download button (%s)", (platform, os) => {
    expect(popupState(facts({ platform, probe: { ok: false, code: "AppMissing" } }))).toEqual({
      kind: "missing",
      os,
    });
  });

  it("outdated for the AppOutdated code, with the versions the app reported", () => {
    const probe = {
      ok: false,
      code: "AppOutdated",
      details: { installed: "0.9.0", required: "1.0.0" },
    } as const;
    expect(popupState(facts({ probe }))).toEqual({
      kind: "outdated",
      installed: "0.9.0",
      required: "1.0.0",
    });
  });

  it("outdated for AppOutdated without details: unknown installed, our minimum required", () => {
    expect(popupState(facts({ probe: { ok: false, code: "AppOutdated" } }))).toEqual({
      kind: "outdated",
      installed: "",
      required: "1.3.0",
    });
  });

  it.each(["Internal", "Timeout", "InvalidRequest", "Busy", "Whatever"])(
    "error for %s, exposing the code",
    (code) => {
      expect(popupState(facts({ probe: { ok: false, code } }))).toEqual({ kind: "error", code });
    },
  );

  it.each(["android", "cros", "openbsd", "fuchsia", ""])(
    "unsupported on platform %j, whatever the probe says",
    (platform) => {
      expect(popupState(facts({ platform })).kind).toBe("unsupported");
      expect(popupState(facts({ platform, probe: null })).kind).toBe("unsupported");
      expect(popupState(facts({ platform, probe: { ok: false, code: "AppMissing" } })).kind).toBe(
        "unsupported",
      );
    },
  );
});
