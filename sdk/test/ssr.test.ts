import { afterEach, describe, expect, it, vi } from "vitest";

// Server-side rendering imports the SDK where there is no window: it must load, stay
// inert and answer "not installed" instead of throwing.
describe("without a window (server-side rendering)", () => {
  afterEach(() => vi.unstubAllGlobals());

  async function loadWithoutWindow() {
    vi.resetModules();
    vi.stubGlobal("window", undefined);
    vi.stubGlobal("location", undefined);
    return import("../src/index");
  }

  it("imports without throwing", async () => {
    await expect(loadWithoutWindow()).resolves.toBeDefined();
  });

  it("status() resolves not installed at once", async () => {
    const sdk = await loadWithoutWindow();
    expect(await sdk.status()).toEqual({
      extension: { installed: false },
      app: { installed: false, outdated: false },
      remembered: false,
      ready: false,
    });
  });

  it("certificates() rejects ExtensionMissing", async () => {
    const sdk = await loadWithoutWindow();
    await expect(sdk.certificates()).rejects.toMatchObject({ code: "ExtensionMissing" });
  });

  it("installUrl() and onChange() work", async () => {
    const sdk = await loadWithoutWindow();
    expect(typeof sdk.installUrl()).toBe("string");
    expect(() => sdk.onChange(() => {})()).not.toThrow();
  });
});
