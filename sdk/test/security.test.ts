import { describe, expect, it, vi } from "vitest";
import { loadSdk, useFakeEnvironment } from "./helpers/env";
import { flush } from "./helpers/fake-script";
import { b64, bytes, needDigest, signResult, wireCertificate } from "./helpers/fixtures";
import { beginSign } from "./helpers/sign";

useFakeEnvironment();

describe("nothing leaks to other frames or the console (T1, T4, T5, T10)", () => {
  async function fullSignature() {
    const env = await loadSdk();
    const log = ["log", "info", "warn", "error", "debug"].map((m) =>
      vi.spyOn(console, m as "log").mockImplementation(() => {}),
    );
    const { prepare, id } = await beginSign(env);
    prepare.mockReturnValue(bytes(32, 0x77));
    env.script.reply(
      id,
      needDigest(1, { certificate: wireCertificate({ displayName: "Ana Beatriz Souza" }) }),
    );
    await flush();
    env.script.reply(id, signResult());
    await flush();
    return { env, log };
  }

  it("every post targets the page's own origin, never '*'", async () => {
    const { env } = await fullSignature();
    expect(env.win.posted.length).toBeGreaterThan(2);
    for (const posted of env.win.posted) expect(posted.targetOrigin).toBe(env.win.origin);
  });

  it("parent and top frames receive nothing", async () => {
    const { env } = await fullSignature();
    expect(env.win.parent.received).toEqual([]);
    expect(env.win.top.received).toEqual([]);
  });

  it("only websign-page frames are posted, and only of kinds discover/request", async () => {
    const { env } = await fullSignature();
    for (const { data } of env.win.posted) {
      const frame = data as { source: string; kind: string };
      expect(frame.source).toBe("websign-page");
      expect(["discover", "request"]).toContain(frame.kind);
    }
  });

  it("the digest is posted once, for the certificate's own request only", async () => {
    const { env } = await fullSignature();
    const needle = b64(bytes(32, 0x77));
    const carrying = env.win.posted.filter((p) => JSON.stringify(p.data).includes(needle));
    expect(carrying).toHaveLength(1);
  });

  it("no personal data or digest reaches the console", async () => {
    const { log } = await fullSignature();
    const text = JSON.stringify(log.flatMap((spy) => spy.mock.calls));
    expect(text).not.toContain("Ana");
    expect(text).not.toContain(b64(bytes(32, 0x77)));
  });

  it("adds no globals while importing or working", async () => {
    const { env } = await fullSignature();
    expect(env.importedGlobals).toEqual([]);
  });

  it("does not expose the certificate list or a way to enumerate the machine (D2)", async () => {
    const env = await loadSdk();
    expect(Object.keys(env.sdk).sort()).toEqual(
      [
        "WebSignError",
        "certificates",
        "fingerprint",
        "installUrl",
        "isWebSignError",
        "onChange",
        "sign",
        "status",
      ].sort(),
    );
  });
});
