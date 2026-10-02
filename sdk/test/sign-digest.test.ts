import { describe, expect, it } from "vitest";
import { loadSdk, useFakeEnvironment } from "./helpers/env";
import { expectError } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { b64, bytes, DIGEST_LENGTH, needDigest, wireCertificate } from "./helpers/fixtures";
import { beginSign, digestsSent } from "./helpers/sign";

useFakeEnvironment();

const HASHES = ["SHA-256", "SHA-384", "SHA-512"] as const;

describe("digest length is exact per hash (T4)", () => {
  for (const hash of HASHES) {
    const length = DIGEST_LENGTH[hash];

    it(`${hash} accepts ${length} bytes`, async () => {
      const env = await loadSdk();
      const { prepare, id } = await beginSign(env, { hash });
      prepare.mockReturnValue(bytes(length, 7));
      env.script.reply(id, needDigest(1, { hash }));
      await flush();
      expect(digestsSent(env)).toEqual([{ seq: 1, digest: b64(bytes(length, 7)) }]);
      expect(env.script.requestsOfType("cancel")).toHaveLength(0);
    });

    for (const wrong of new Set([
      0,
      1,
      20,
      length - 1,
      length + 1,
      ...Object.values(DIGEST_LENGTH),
    ])) {
      if (wrong === length) continue;
      it(`${hash} rejects ${wrong} bytes: cancel + InvalidRequest, nothing sent`, async () => {
        const env = await loadSdk();
        const { s, prepare, id } = await beginSign(env, { hash });
        prepare.mockReturnValue(bytes(wrong));
        env.script.reply(id, needDigest(1, { hash }));
        await flush();
        expect(digestsSent(env)).toEqual([]);
        expect(env.script.only("cancel").id).toBe(id);
        const error = expectError(env, s.outcome(), "InvalidRequest");
        expect(error.message).toBe(`Digest is ${wrong} bytes; ${hash} requires ${length}.`);
      });
    }
  }

  it("uses the documented message for a SHA-1 sized digest", async () => {
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env);
    prepare.mockReturnValue(bytes(20));
    env.script.reply(id, needDigest(1));
    await flush();
    expect(expectError(env, s.outcome(), "InvalidRequest").message).toBe(
      "Digest is 20 bytes; SHA-256 requires 32.",
    );
  });
});

describe("prepare results are coerced to bytes", () => {
  it("accepts an ArrayBuffer", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    prepare.mockReturnValue(bytes(32, 9).buffer as ArrayBuffer);
    env.script.reply(id, needDigest(1));
    await flush();
    expect(digestsSent(env)).toEqual([{ seq: 1, digest: b64(bytes(32, 9)) }]);
  });

  it("sends only the view's bytes, not the whole backing buffer", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    const backing = new Uint8Array(100).fill(0xee);
    backing.set(bytes(32, 0x01), 10);
    prepare.mockReturnValue(backing.subarray(10, 42));
    env.script.reply(id, needDigest(1));
    await flush();
    expect(digestsSent(env)).toEqual([{ seq: 1, digest: b64(bytes(32, 0x01)) }]);
  });

  for (const [name, value] of [
    ["undefined", undefined],
    ["null", null],
    ["a string", "a".repeat(32)],
    ["an array of numbers", new Array(32).fill(1)],
    ["an array of digests (no batch, D4)", [bytes(32), bytes(32)]],
    ["a number", 32],
  ] as const) {
    it(`rejects ${name} with cancel + InvalidRequest`, async () => {
      const env = await loadSdk();
      const { s, prepare, id } = await beginSign(env);
      prepare.mockReturnValue(value as never);
      env.script.reply(id, needDigest(1));
      await flush();
      expect(digestsSent(env)).toEqual([]);
      expect(env.script.only("cancel").id).toBe(id);
      expectError(env, s.outcome(), "InvalidRequest");
    });
  }
});

describe("prepare failures", () => {
  it("a throw sends cancel and rejects Aborted carrying the original error as cause", async () => {
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env);
    const boom = new Error("boom");
    prepare.mockImplementation(() => {
      throw boom;
    });
    env.script.reply(id, needDigest(1));
    await flush();
    expect(env.script.only("cancel").id).toBe(id);
    expect(digestsSent(env)).toEqual([]);
    expect(expectError(env, s.outcome(), "Aborted").cause).toBe(boom);
  });

  it("a rejected promise behaves like a throw", async () => {
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env);
    const boom = new Error("async boom");
    prepare.mockRejectedValue(boom);
    env.script.reply(id, needDigest(1));
    await flush();
    expect(env.script.only("cancel").id).toBe(id);
    expect(expectError(env, s.outcome(), "Aborted").cause).toBe(boom);
  });

  it("does not leak the prepare error message into the rejection message", async () => {
    // Errors from site code may carry document data; the SDK message stays generic.
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env);
    prepare.mockRejectedValue(new Error("contract of Ana Souza CPF 123.456.789-09"));
    env.script.reply(id, needDigest(1));
    await flush();
    expect(expectError(env, s.outcome(), "Aborted").message).not.toContain("Ana");
  });

  it("a late error reply after the SDK cancelled is ignored", async () => {
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env);
    prepare.mockReturnValue(bytes(5));
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(id, { type: "error", code: "Aborted", message: "x" });
    await flush();
    expectError(env, s.outcome(), "InvalidRequest");
  });
});

describe("need_digest is checked against what was asked (T4)", () => {
  // A need_digest whose hash or algorithm differs from the request would make the site
  // hash the wrong thing; the SDK refuses before calling prepare (SPEC §6).
  it("hash different from the request: cancel + InvalidRequest, prepare not called", async () => {
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env, { hash: "SHA-256" });
    env.script.reply(id, needDigest(1, { hash: "SHA-512" }));
    await flush();
    expect(prepare).not.toHaveBeenCalled();
    expect(env.script.only("cancel").id).toBe(id);
    expectError(env, s.outcome(), "InvalidRequest");
  });

  it("algorithm outside the requested list: cancel + InvalidRequest", async () => {
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env, { algorithm: ["ECDSA"] });
    env.script.reply(id, needDigest(1, { algorithm: "RSASSA-PSS" }));
    await flush();
    expect(prepare).not.toHaveBeenCalled();
    expect(env.script.only("cancel").id).toBe(id);
    expectError(env, s.outcome(), "InvalidRequest");
  });

  it("algorithm the SDK does not know: cancel + InvalidRequest", async () => {
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env);
    env.script.reply(id, needDigest(1, { algorithm: "EdDSA" }));
    await flush();
    expect(prepare).not.toHaveBeenCalled();
    expectError(env, s.outcome(), "InvalidRequest");
  });

  it("malformed need_digest (bad seq, missing certificate) is refused without calling prepare", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    for (const bad of [
      needDigest(-1),
      needDigest(1.5),
      needDigest("1" as never),
      { ...needDigest(1), certificate: undefined },
    ]) {
      env.script.reply(id, bad);
    }
    await flush();
    expect(prepare).not.toHaveBeenCalled();
    expect(digestsSent(env)).toEqual([]);
  });

  it("a need_digest whose certificate cannot be read: cancel + Internal, prepare not called", async () => {
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env);
    env.script.reply(
      id,
      needDigest(1, { certificate: { ...wireCertificate(), der: "not base64!" } }),
    );
    await flush();
    expect(prepare).not.toHaveBeenCalled();
    expect(env.script.only("cancel").id).toBe(id);
    expectError(env, s.outcome(), "Internal");
  });

  it("a repeated or older seq is ignored: prepare runs once per certificate", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    env.script.reply(id, needDigest(2));
    await flush();
    env.script.reply(id, needDigest(2));
    env.script.reply(id, needDigest(1));
    await flush();
    expect(prepare).toHaveBeenCalledTimes(1);
    expect(digestsSent(env).map((d) => d.seq)).toEqual([2]);
  });
});
