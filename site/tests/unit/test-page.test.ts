import { describe, expect, test } from "bun:test";
import { type Outcome, setupState, summaryText } from "../../src/client/test-page/state.ts";

const installed = {
  extension: { installed: true, version: "0.2.0" },
  app: { installed: true, version: "0.2.0", outdated: false },
};

describe("setupState", () => {
  test("null is still looking", () => expect(setupState(null)).toBe("checking"));
  test("no extension", () => {
    expect(
      setupState({ extension: { installed: false }, app: { installed: false, outdated: false } }),
    ).toBe("extension-missing");
  });
  test("extension without the app", () => {
    expect(
      setupState({ extension: { installed: true }, app: { installed: false, outdated: false } }),
    ).toBe("app-missing");
  });
  test("an old app", () => {
    expect(setupState({ ...installed, app: { installed: true, outdated: true } })).toBe("outdated");
  });
  test("everything there", () => expect(setupState(installed)).toBe("ready"));
});

describe("summaryText", () => {
  test("a success names the algorithm, the key type and the browser's check, never a person", () => {
    const outcome: Outcome = {
      hash: "SHA-256",
      algorithm: "ECDSA",
      key: { type: "EC", curve: "P-256" },
      check: "verified",
    };
    expect(summaryText(installed, outcome)).toBe(
      [
        "signlocal test",
        "setup: ready",
        "extension: 0.2.0",
        "app: 0.2.0",
        "result: success",
        "error: none",
        "hash: SHA-256",
        "algorithm: ECDSA",
        "key: EC P-256",
        "browser check: verified",
      ].join("\n"),
    );
  });

  test("an invalid signature is a failure", () => {
    const outcome: Outcome = {
      hash: "SHA-256",
      algorithm: "RSASSA-PSS",
      key: { type: "RSA", bits: 2048 },
      check: "invalid",
    };
    expect(summaryText(installed, outcome)).toContain("result: failure");
    expect(summaryText(installed, outcome)).toContain("key: RSA 2048");
  });

  test("an unsupported key says it was not verified", () => {
    const outcome: Outcome = {
      hash: "SHA-384",
      algorithm: "ECDSA",
      key: { type: "EC", curve: "P-384" },
      check: "unsupported",
    };
    expect(summaryText(installed, outcome)).toContain("browser check: not verified");
  });

  test("an error carries its code and nothing else", () => {
    expect(summaryText(null, { hash: "SHA-256", error: "UserCancelled" })).toBe(
      [
        "signlocal test",
        "setup: checking",
        "result: failure",
        "error: UserCancelled",
        "hash: SHA-256",
      ].join("\n"),
    );
  });

  test("before any attempt there is only the setup", () => {
    expect(summaryText(installed, null)).toBe(
      ["signlocal test", "setup: ready", "extension: 0.2.0", "app: 0.2.0"].join("\n"),
    );
  });
});
