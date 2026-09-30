import { describe, expect, it } from "vitest";
import { WebSign, WebSignError } from "../src/index";
import { APP, CERTIFICATE, fake, helloRule, send } from "./support";

async function connected(rules: Parameters<typeof fake>[0]["rules"]) {
  const app = fake({ rules: [helloRule(), ...rules] });
  return { app, websign: await WebSign.connect({ executable: app.executable }) };
}

describe("status", () => {
  it("returns app info and whether the caller is remembered", async () => {
    const { app, websign } = await connected([
      { on: "status", do: [send({ type: "status", app: APP, remembered: true })] },
    ]);
    expect(await websign.status()).toEqual({ app: APP, remembered: true });
    await websign.close();
    const request = app.received()[1];
    expect(request).toEqual({ v: 1, id: "n2", type: "status" });
  });

  it("rejects an error reply with the app's code", async () => {
    const { websign } = await connected([
      { on: "status", do: [send({ type: "error", code: "Busy", message: "queue full" })] },
    ]);
    await expect(websign.status()).rejects.toMatchObject({ name: "WebSignError", code: "Busy" });
    await websign.close();
  });

  it("rejects an unexpected reply type with Internal", async () => {
    const { websign } = await connected([{ on: "status", do: [send({ type: "done" })] }]);
    await expect(websign.status()).rejects.toMatchObject({ code: "Internal" });
    await websign.close();
  });
});

describe("certificates", () => {
  const reply = {
    on: "choose",
    repeat: true,
    do: [send({ type: "choose.result", certificates: [CERTIFICATE] })],
  };

  it("sends choose without a filter by default", async () => {
    const { app, websign } = await connected([reply]);
    const [certificate, ...others] = await websign.certificates();
    expect(others).toEqual([]);
    expect(certificate).toMatchObject({ fingerprint: CERTIFICATE.fingerprint, chain: [] });
    await websign.close();
    expect(app.received()[1]).toEqual({ v: 1, id: "n2", type: "choose" });
  });

  it("deduplicates algorithms and keeps their order", async () => {
    const { app, websign } = await connected([reply]);
    await websign.certificates({ algorithms: ["ECDSA", "RSASSA-PSS", "ECDSA"] });
    await websign.close();
    expect(app.received()[1]?.filter).toEqual({ algorithms: ["ECDSA", "RSASSA-PSS"] });
  });

  it("omits an empty filter (the protocol refuses empty algorithm lists)", async () => {
    const { app, websign } = await connected([reply]);
    await websign.certificates({});
    await websign.certificates({ algorithms: [] });
    await websign.close();
    expect(
      app
        .received()
        .slice(1)
        .map((m) => "filter" in m),
    ).toEqual([false, false]);
  });

  it("maps UserCancelled and NoCertificates errors", async () => {
    for (const code of ["UserCancelled", "NoCertificates"]) {
      const { websign } = await connected([
        { on: "choose", do: [send({ type: "error", code, message: "no" })] },
      ]);
      const error = await websign.certificates().catch((e: unknown) => e);
      expect(error).toBeInstanceOf(WebSignError);
      expect((error as WebSignError).code).toBe(code);
      await websign.close();
    }
  });

  it("rejects a reply without a certificate list", async () => {
    const { websign } = await connected([
      { on: "choose", do: [send({ type: "choose.result", certificates: "x" })] },
    ]);
    await expect(websign.certificates()).rejects.toMatchObject({ code: "Internal" });
    await websign.close();
  });
});

describe("openDiagnostics", () => {
  it("sends the tab and resolves on done", async () => {
    const { app, websign } = await connected([
      { on: "diagnostics.open", repeat: true, do: [send({ type: "done" })] },
    ]);
    await websign.openDiagnostics("devices");
    await websign.openDiagnostics();
    await websign.close();
    const [, first, second] = app.received();
    expect(first).toMatchObject({ type: "diagnostics.open", tab: "devices" });
    expect(second).toEqual({ v: 1, id: "n3", type: "diagnostics.open" });
  });
});

describe("concurrent requests", () => {
  it("routes replies to the right caller by id, in any order", async () => {
    const { websign } = await connected([
      {
        on: "status",
        do: [{ delay: 60 }, send({ type: "status", app: APP, remembered: false })],
      },
      { on: "diagnostics.open", do: [send({ type: "done" })] },
    ]);
    const slow = websign.status();
    await websign.openDiagnostics();
    expect((await slow).remembered).toBe(false);
    await websign.close();
  });
});
