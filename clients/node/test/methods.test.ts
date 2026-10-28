import { describe, expect, it } from "vitest";
import { Signlocal, SignlocalError } from "../src/index";
import { APP, CERTIFICATE, eventually, fake, helloRule, send } from "./support";

async function connected(rules: Parameters<typeof fake>[0]["rules"]) {
  const app = fake({ rules: [helloRule(), ...rules] });
  return { app, signlocal: await Signlocal.connect({ executable: app.executable }) };
}

describe("status", () => {
  it("returns app info and whether the caller is remembered", async () => {
    const { app, signlocal } = await connected([
      { on: "status", do: [send({ type: "status", app: APP, remembered: true })] },
    ]);
    expect(await signlocal.status()).toEqual({ app: APP, remembered: true });
    await signlocal.close();
    const request = app.received()[1];
    expect(request).toEqual({ v: 1, id: "n2", type: "status" });
  });

  it("rejects an error reply with the app's code", async () => {
    const { signlocal } = await connected([
      { on: "status", do: [send({ type: "error", code: "Busy", message: "queue full" })] },
    ]);
    await expect(signlocal.status()).rejects.toMatchObject({
      name: "SignlocalError",
      code: "Busy",
    });
    await signlocal.close();
  });

  it("rejects an unexpected reply type with Internal", async () => {
    const { signlocal } = await connected([{ on: "status", do: [send({ type: "done" })] }]);
    await expect(signlocal.status()).rejects.toMatchObject({ code: "Internal" });
    await signlocal.close();
  });
});

describe("certificates", () => {
  const reply = {
    on: "choose",
    repeat: true,
    do: [send({ type: "choose.result", certificates: [CERTIFICATE] })],
  };

  it("sends choose without a filter by default", async () => {
    const { app, signlocal } = await connected([reply]);
    const [certificate, ...others] = await signlocal.certificates();
    expect(others).toEqual([]);
    expect(certificate).toMatchObject({ fingerprint: CERTIFICATE.fingerprint, chain: [] });
    await signlocal.close();
    expect(app.received()[1]).toEqual({ v: 1, id: "n2", type: "choose" });
  });

  it("deduplicates algorithms and keeps their order", async () => {
    const { app, signlocal } = await connected([reply]);
    await signlocal.certificates({ algorithm: ["ECDSA", "RSASSA-PSS", "ECDSA"] });
    await signlocal.certificates({ algorithm: "ECDSA" });
    await signlocal.close();
    expect(app.received()[1]?.filter).toEqual({ algorithms: ["ECDSA", "RSASSA-PSS"] });
    expect(app.received()[2]?.filter).toEqual({ algorithms: ["ECDSA"] });
  });

  it("sends no filter without an algorithm", async () => {
    const { app, signlocal } = await connected([reply]);
    await signlocal.certificates({});
    await signlocal.close();
    expect(app.received()[1]).toEqual({ v: 1, id: "n2", type: "choose" });
  });

  it("rejects InvalidRequest and sends nothing for an empty or unknown algorithm", async () => {
    const { app, signlocal } = await connected([reply]);
    for (const algorithm of [[], "DSA", ["ECDSA", "DSA"]]) {
      await expect(signlocal.certificates({ algorithm } as never)).rejects.toMatchObject({
        code: "InvalidRequest",
      });
    }
    await signlocal.close();
    expect(app.received()).toHaveLength(1);
  });

  it("rejects NoCertificates on an empty choice", async () => {
    const { signlocal } = await connected([
      { on: "choose", do: [send({ type: "choose.result", certificates: [] })] },
    ]);
    await expect(signlocal.certificates()).rejects.toMatchObject({ code: "NoCertificates" });
    await signlocal.close();
  });

  it("sends cancel and rejects Aborted when the signal fires", async () => {
    const { app, signlocal } = await connected([{ on: "choose", do: [] }]);
    const controller = new AbortController();
    const promise = signlocal.certificates({ signal: controller.signal });
    await eventually(() => app.received().some((m) => m.type === "choose"));
    const reason = new Error("stop");
    controller.abort(reason);
    await expect(promise).rejects.toMatchObject({ code: "Aborted", cause: reason });
    await eventually(() => app.received().some((m) => m.type === "cancel"));
    await signlocal.close();
    await expect(signlocal.certificates({ signal: AbortSignal.abort() })).rejects.toMatchObject({
      code: "Aborted",
    });
  });

  it("maps UserCancelled and NoCertificates errors", async () => {
    for (const code of ["UserCancelled", "NoCertificates"]) {
      const { signlocal } = await connected([
        { on: "choose", do: [send({ type: "error", code, message: "no" })] },
      ]);
      const error = await signlocal.certificates().catch((e: unknown) => e);
      expect(error).toBeInstanceOf(SignlocalError);
      expect((error as SignlocalError).code).toBe(code);
      await signlocal.close();
    }
  });

  it("rejects a reply without a certificate list", async () => {
    const { signlocal } = await connected([
      { on: "choose", do: [send({ type: "choose.result", certificates: "x" })] },
    ]);
    await expect(signlocal.certificates()).rejects.toMatchObject({ code: "Internal" });
    await signlocal.close();
  });
});

describe("openDiagnostics", () => {
  it("sends the tab and resolves on done", async () => {
    const { app, signlocal } = await connected([
      { on: "diagnostics.open", repeat: true, do: [send({ type: "done" })] },
    ]);
    await signlocal.openDiagnostics("devices");
    await signlocal.openDiagnostics();
    await signlocal.close();
    const [, first, second] = app.received();
    expect(first).toMatchObject({ type: "diagnostics.open", tab: "devices" });
    expect(second).toEqual({ v: 1, id: "n3", type: "diagnostics.open" });
  });
});

describe("concurrent requests", () => {
  it("routes replies to the right caller by id, in any order", async () => {
    const { signlocal } = await connected([
      {
        on: "status",
        do: [{ delay: 60 }, send({ type: "status", app: APP, remembered: false })],
      },
      { on: "diagnostics.open", do: [send({ type: "done" })] },
    ]);
    const slow = signlocal.status();
    await signlocal.openDiagnostics();
    expect((await slow).remembered).toBe(false);
    await signlocal.close();
  });
});
