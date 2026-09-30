import { describe, expect, it, vi } from "vitest";
import { isTestOrigin } from "../../src/testing/guard";
import { load } from "./dom";

describe("@websign/sdk/testing cannot ship by accident", () => {
  const local = [
    "http://localhost:5173/",
    "http://127.0.0.1:8080/",
    "http://[::1]:3000/",
    "https://app.localhost/",
    "https://shop.test/",
    "https://demo.example/",
    "file:///tmp/index.html",
  ];
  for (const url of local) {
    it(`installs on ${url}`, async () => {
      const { testing } = await load(url);
      await expect(testing.installFakeWebSign()).resolves.toBeDefined();
    });
  }

  for (const url of ["https://bank.com/", "https://localhost.evil.com/", "http://10.0.0.8/"]) {
    it(`refuses ${url} unless allowAnyOrigin`, async () => {
      const { testing } = await load(url);
      await expect(testing.installFakeWebSign()).rejects.toThrow(/public test keys/);
      await expect(testing.installFakeWebSign({ allowAnyOrigin: true })).resolves.toBeDefined();
    });
  }

  it("about:blank and test DOMs without a host count as local", () => {
    expect(isTestOrigin({ protocol: "about:", hostname: "" })).toBe(true);
    expect(isTestOrigin(undefined)).toBe(true);
  });

  it("warns on the console at every install", async () => {
    const { testing, warn } = await load();
    await testing.installFakeWebSign();
    await testing.installFakeWebSign();
    expect(warn).toHaveBeenCalledTimes(2);
    expect(String(warn.mock.calls[0]?.[0])).toMatch(/FAKE WebeSign.*Never ship/s);
  });

  it("warns once when a real extension answers on the same page", async () => {
    const { win, testing, warn } = await load();
    await testing.installFakeWebSign();
    const real = { source: "websign-extension", kind: "announce", extension: {}, protocols: {} };
    win.postMessage(real, win.location.origin);
    win.postMessage(real, win.location.origin);
    await vi.waitFor(() => expect(warn).toHaveBeenCalledTimes(2));
    await new Promise((resolve) => setTimeout(resolve, 10));
    expect(String(warn.mock.calls[1]?.[0])).toMatch(/real WebeSign extension/);
    expect(warn).toHaveBeenCalledTimes(2);
  });

  it("needs a window", async () => {
    vi.resetModules();
    vi.stubGlobal("window", undefined);
    const testing = await import("../../src/testing/index");
    await expect(testing.installFakeWebSign()).rejects.toThrow(/needs a window/);
    vi.unstubAllGlobals();
  });

  it("is not reachable from the main entry", async () => {
    const { sdk } = await load();
    expect(Object.keys(sdk)).not.toContain("installFakeWebSign");
  });
});
