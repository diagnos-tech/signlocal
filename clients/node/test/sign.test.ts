import { describe, expect, it, vi } from "vitest";
import { WebSign, WebSignError } from "../src/index";
import { CERTIFICATE, eventually, fake, helloRule, isAlive, type Rule, send } from "./support";

const digest32 = new Uint8Array(32).fill(7);
const b64 = (bytes: Uint8Array) => Buffer.from(bytes).toString("base64");

const needDigest = (seq: number, hash = "SHA-256", algorithm = "RSASSA-PKCS1-v1_5") =>
  send({
    type: "sign.need_digest",
    seq,
    hash,
    algorithm,
    certificate: { ...CERTIFICATE, fingerprint: `${seq}`.repeat(64) },
  });

const result = send({
  type: "sign.result",
  hash: "SHA-256",
  algorithm: "RSASSA-PKCS1-v1_5",
  certificate: CERTIFICATE,
  signature: b64(new Uint8Array([1, 2, 3, 4])),
});

async function connected(rules: Rule[]) {
  const app = fake({ rules: [helloRule(), ...rules] });
  return { app, websign: await WebSign.connect({ executable: app.executable }) };
}

const failure = async (promise: Promise<unknown>) => {
  const error = await promise.then(
    () => undefined,
    (e: unknown) => e,
  );
  expect(error).toBeInstanceOf(WebSignError);
  return error as WebSignError;
};

const happyRules: Rule[] = [
  { on: "sign.begin", do: [needDigest(1)] },
  { on: "sign.digest", do: [result] },
];

describe("sign", () => {
  it("asks prepare for the digest of the chosen certificate and returns decoded bytes", async () => {
    const { app, websign } = await connected(happyRules);
    const prepare = vi.fn(() => digest32);
    const signed = await websign.sign({ hash: "SHA-256", prepare });
    await websign.close();

    expect(prepare).toHaveBeenCalledTimes(1);
    expect(prepare).toHaveBeenCalledWith(
      expect.objectContaining({ displayName: "Ana Beatriz Souza" }),
      { hash: "SHA-256", algorithm: "RSASSA-PKCS1-v1_5" },
    );
    expect(signed.signature).toBeInstanceOf(Uint8Array);
    expect([...signed.signature]).toEqual([1, 2, 3, 4]);
    expect(signed).toMatchObject({ hash: "SHA-256", algorithm: "RSASSA-PKCS1-v1_5" });
    expect(signed.certificate.fingerprint).toBe(CERTIFICATE.fingerprint);

    const [, begin, digest] = app.received();
    expect(begin).toEqual({ v: 1, id: "n2", type: "sign.begin", hash: "SHA-256" });
    expect(digest).toEqual({ v: 1, id: "n2", type: "sign.digest", seq: 1, digest: b64(digest32) });
  });

  it("forwards algorithms (deduplicated) and the preselected fingerprint", async () => {
    const { app, websign } = await connected(happyRules);
    await websign.sign({
      hash: "SHA-256",
      algorithm: ["ECDSA", "RSASSA-PKCS1-v1_5", "ECDSA"],
      certificate: "ab".repeat(32),
      prepare: () => digest32,
    });
    await websign.close();
    expect(app.received()[1]).toMatchObject({
      algorithms: ["ECDSA", "RSASSA-PKCS1-v1_5"],
      certificate: "ab".repeat(32),
    });
  });

  it("accepts a single algorithm and a Certificate object to preselect", async () => {
    const { app, websign } = await connected([
      { on: "choose", do: [send({ type: "choose.result", certificates: [CERTIFICATE] })] },
      ...happyRules,
    ]);
    const [chosen] = await websign.certificates();
    if (chosen === undefined) throw new Error("no certificate");
    await websign.sign({
      hash: "SHA-256",
      algorithm: "RSASSA-PKCS1-v1_5",
      certificate: chosen,
      prepare: () => digest32,
    });
    await websign.close();
    const begin = app.received().find((m) => m.type === "sign.begin");
    expect(begin).toEqual({
      v: 1,
      id: "n3",
      type: "sign.begin",
      hash: "SHA-256",
      algorithms: ["RSASSA-PKCS1-v1_5"],
      certificate: CERTIFICATE.fingerprint,
    });
  });

  it("accepts an async prepare and an ArrayBuffer result", async () => {
    const { websign } = await connected(happyRules);
    const signed = await websign.sign({
      hash: "SHA-256",
      prepare: async () => digest32.slice().buffer,
    });
    expect(signed.signature.length).toBe(4);
    await websign.close();
  });

  it.each([
    ["SHA-384", 48],
    ["SHA-512", 64],
  ] as const)("requires %s digests of %i bytes", async (hash, length) => {
    const { app, websign } = await connected([
      { on: "sign.begin", do: [needDigest(1, hash)] },
      { on: "sign.digest", do: [result] },
    ]);
    await websign.sign({ hash, prepare: () => new Uint8Array(length) });
    await websign.close();
    expect(app.received()[2]).toMatchObject({ type: "sign.digest" });
  });

  it("runs prepare again for a switched certificate and sends both digests", async () => {
    const { app, websign } = await connected([
      { on: "sign.begin", do: [needDigest(1)] },
      { on: "sign.digest", where: { seq: 1 }, do: [needDigest(2)] },
      { on: "sign.digest", where: { seq: 2 }, do: [result] },
    ]);
    const seen: string[] = [];
    await websign.sign({
      hash: "SHA-256",
      prepare: (certificate) => {
        seen.push(certificate.fingerprint.slice(0, 1));
        return digest32;
      },
    });
    await websign.close();
    expect(seen).toEqual(["1", "2"]);
    expect(
      app
        .received()
        .filter((m) => m.type === "sign.digest")
        .map((m) => m.seq),
    ).toEqual([1, 2]);
  });

  it("discards a stale digest when a newer request arrives while prepare runs", async () => {
    const { app, websign } = await connected([
      { on: "sign.begin", do: [needDigest(1), { delay: 20 }, needDigest(2)] },
      { on: "sign.digest", do: [result] },
    ]);
    let release: () => void = () => {};
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    let calls = 0;
    const promise = websign.sign({
      hash: "SHA-256",
      prepare: async () => {
        calls += 1;
        if (calls === 1) await gate;
        return digest32;
      },
    });
    await eventually(() => calls === 2);
    release();
    await promise;
    await websign.close();
    const digests = app.received().filter((m) => m.type === "sign.digest");
    expect(digests.map((m) => m.seq)).toEqual([2]);
  });

  it("ignores a stale prepare failure", async () => {
    const { websign } = await connected([
      { on: "sign.begin", do: [needDigest(1), { delay: 20 }, needDigest(2)] },
      { on: "sign.digest", do: [result] },
    ]);
    let calls = 0;
    const signed = await websign.sign({
      hash: "SHA-256",
      prepare: async () => {
        calls += 1;
        if (calls === 1) {
          await new Promise((resolve) => setTimeout(resolve, 100));
          throw new Error("stale");
        }
        return digest32;
      },
    });
    expect(signed.signature.length).toBe(4);
    await websign.close();
  });

  it.each([
    "UserCancelled",
    "Timeout",
    "PinLocked",
    "TokenRemoved",
    "CertificateNotValid",
    "UnsupportedAlgorithm",
    "DriverFailure",
  ])("rejects with the app's %s", async (code) => {
    const { websign } = await connected([
      {
        on: "sign.begin",
        do: [send({ type: "error", code, message: "no", details: { native: "X" } })],
      },
    ]);
    const error = await failure(websign.sign({ hash: "SHA-256", prepare: () => digest32 }));
    expect(error.code).toBe(code);
    expect(error.details).toEqual({ native: "X" });
    await websign.close();
  });

  it("rejects with Internal on a malformed signature", async () => {
    const { websign } = await connected([
      { on: "sign.begin", do: [needDigest(1)] },
      { on: "sign.digest", do: [send({ ...(result.send as object), signature: "not base64!" })] },
    ]);
    expect((await failure(websign.sign({ hash: "SHA-256", prepare: () => digest32 }))).code).toBe(
      "Internal",
    );
    await websign.close();
  });
});

describe("sign input errors", () => {
  it("rejects with InvalidRequest and sends nothing for bad options", async () => {
    const { app, websign } = await connected([]);
    const prepare = () => digest32;
    const bad = [
      { hash: "MD5", prepare },
      { hash: "SHA-256" },
      { hash: "SHA-256", prepare, algorithm: [] },
      { hash: "SHA-256", prepare, algorithm: ["DSA"] },
      { hash: "SHA-256", prepare, algorithm: "DSA" },
      { hash: "SHA-256", prepare, certificate: "AB".repeat(32) },
      { hash: "SHA-256", prepare, certificate: { fingerprint: "x" } },
    ];
    for (const options of bad) {
      const error = await failure(websign.sign(options as never));
      expect(error.code).toBe("InvalidRequest");
    }
    await websign.close();
    expect(app.received()).toHaveLength(1);
  });

  it("cancels and rejects InvalidRequest when the digest has the wrong length", async () => {
    const { app, websign } = await connected([
      { on: "sign.begin", do: [needDigest(1)] },
      { on: "cancel", do: [send({ type: "error", code: "Aborted", message: "cancelled" })] },
    ]);
    const error = await failure(
      websign.sign({ hash: "SHA-256", prepare: () => new Uint8Array(20) }),
    );
    expect(error.code).toBe("InvalidRequest");
    expect(error.message).toBe("digest is 20 bytes; SHA-256 requires 32");
    await websign.close();
    expect(app.received().map((m) => m.type)).toEqual(["hello", "sign.begin", "cancel"]);
  });

  it("cancels and rejects Aborted with the cause when prepare throws", async () => {
    const { app, websign } = await connected([{ on: "sign.begin", do: [needDigest(1)] }]);
    const boom = new Error("disk on fire");
    const error = await failure(
      websign.sign({
        hash: "SHA-256",
        prepare: () => {
          throw boom;
        },
      }),
    );
    expect(error.code).toBe("Aborted");
    expect(error.cause).toBe(boom);
    await websign.close();
    expect(app.received().at(-1)).toMatchObject({ type: "cancel", id: "n2" });
  });

  it.each([
    ["another hash", needDigest(1, "SHA-384")],
    ["an algorithm outside the requested set", needDigest(1, "SHA-256", "RSASSA-PSS")],
  ])("cancels with InvalidRequest, before prepare, when the app asks for %s", async (_, need) => {
    const { app, websign } = await connected([{ on: "sign.begin", do: [need] }]);
    const prepare = vi.fn(() => digest32);
    const error = await failure(
      websign.sign({
        hash: "SHA-256",
        algorithm: ["RSASSA-PKCS1-v1_5", "ECDSA"],
        prepare,
      }),
    );
    expect(error.code).toBe("InvalidRequest");
    expect(prepare).not.toHaveBeenCalled();
    await eventually(() => app.received().some((m) => m.type === "cancel"));
    await websign.close();
    expect(app.received().map((m) => m.type)).toEqual(["hello", "sign.begin", "cancel"]);
  });

  it("accepts any known algorithm when none was requested", async () => {
    const { websign } = await connected([
      { on: "sign.begin", do: [needDigest(1, "SHA-256", "RSASSA-PSS")] },
      { on: "sign.digest", do: [result] },
    ]);
    const prepare = vi.fn(() => digest32);
    await websign.sign({ hash: "SHA-256", prepare });
    await websign.close();
    expect(prepare).toHaveBeenCalledWith(expect.anything(), {
      hash: "SHA-256",
      algorithm: "RSASSA-PSS",
    });
  });

  it("rejects Internal when the app's digest request is malformed", async () => {
    const { app, websign } = await connected([
      { on: "sign.begin", do: [send({ type: "sign.need_digest", seq: "one" })] },
    ]);
    expect((await failure(websign.sign({ hash: "SHA-256", prepare: () => digest32 }))).code).toBe(
      "Internal",
    );
    await websign.close();
    expect(app.received().at(-1)).toMatchObject({ type: "cancel" });
  });
});

describe("AbortSignal", () => {
  it("rejects Aborted without sending anything when already aborted", async () => {
    const { app, websign } = await connected([]);
    const reason = new Error("stop");
    const error = await failure(
      websign.sign({ hash: "SHA-256", prepare: () => digest32, signal: AbortSignal.abort(reason) }),
    );
    expect(error.code).toBe("Aborted");
    expect(error.cause).toBe(reason);
    await websign.close();
    expect(app.received()).toHaveLength(1);
  });

  it("sends cancel and rejects Aborted at once when aborted mid-flight", async () => {
    const { app, websign } = await connected([{ on: "sign.begin", do: [] }]);
    const controller = new AbortController();
    const promise = websign.sign({
      hash: "SHA-256",
      prepare: () => digest32,
      signal: controller.signal,
    });
    await eventually(() => app.received().some((m) => m.type === "sign.begin"));
    controller.abort();
    expect((await failure(promise)).code).toBe("Aborted");
    await eventually(() => app.received().some((m) => m.type === "cancel"));
    expect(app.received().at(-1)).toMatchObject({ type: "cancel", id: "n2" });
    await websign.close();
  });

  it("aborts while prepare is running and never sends its late digest", async () => {
    const { app, websign } = await connected([{ on: "sign.begin", do: [needDigest(1)] }]);
    const controller = new AbortController();
    let release: () => void = () => {};
    const gate = new Promise<void>((resolve) => {
      release = resolve;
    });
    let started = false;
    const promise = websign.sign({
      hash: "SHA-256",
      signal: controller.signal,
      prepare: async () => {
        started = true;
        await gate;
        return digest32;
      },
    });
    await eventually(() => started);
    controller.abort();
    expect((await failure(promise)).code).toBe("Aborted");
    release();
    await new Promise((resolve) => setTimeout(resolve, 50));
    await websign.close();
    expect(app.received().map((m) => m.type)).toEqual(["hello", "sign.begin", "cancel"]);
  });

  it("is not affected by an abort after completion", async () => {
    const { websign } = await connected(happyRules);
    const controller = new AbortController();
    await websign.sign({ hash: "SHA-256", prepare: () => digest32, signal: controller.signal });
    controller.abort();
    await websign.close();
  });

  it("leaves no abort listener behind", async () => {
    const { websign } = await connected(happyRules);
    const controller = new AbortController();
    const remove = vi.spyOn(controller.signal, "removeEventListener");
    await websign.sign({ hash: "SHA-256", prepare: () => digest32, signal: controller.signal });
    expect(remove).toHaveBeenCalledWith("abort", expect.any(Function));
    await websign.close();
  });
});

describe("process lifetime", () => {
  it("rejects a sign in flight when the app dies, and leaves no process behind", async () => {
    const { app, websign } = await connected([
      { on: "sign.begin", do: [needDigest(1), { exit: 4 }] },
    ]);
    const error = await failure(websign.sign({ hash: "SHA-256", prepare: () => digest32 }));
    expect(error.code).toBe("Internal");
    await websign.close();
    expect(isAlive(app.pid())).toBe(false);
  });

  it("does not spawn a second process for concurrent signs", async () => {
    const { app, websign } = await connected([
      { on: "sign.begin", repeat: true, do: [needDigest(1)] },
      { on: "sign.digest", repeat: true, do: [result] },
    ]);
    const both = await Promise.all([
      websign.sign({ hash: "SHA-256", prepare: () => digest32 }),
      websign.sign({ hash: "SHA-256", prepare: () => digest32 }),
    ]);
    expect(both).toHaveLength(2);
    await websign.close();
    expect(
      app
        .received()
        .filter((m) => m.type === "sign.begin")
        .map((m) => m.id),
    ).toEqual(["n2", "n3"]);
  });
});
