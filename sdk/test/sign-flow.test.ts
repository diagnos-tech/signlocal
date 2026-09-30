import { describe, expect, it } from "vitest";
import { loadSdk, useFakeEnvironment } from "./helpers/env";
import { expectError, expectValue } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import {
  b64,
  bytes,
  ERROR_CODES,
  FINGERPRINT,
  needDigest,
  signResult,
  wireCertificate,
  wireError,
} from "./helpers/fixtures";
import { beginSign, digestsSent } from "./helpers/sign";

useFakeEnvironment();

describe("sign(): begin -> need_digest -> prepare -> digest -> result (D1)", () => {
  it("sends sign.begin with only the hash when nothing else is given", async () => {
    const env = await loadSdk();
    await beginSign(env);
    const message = env.script.only("sign.begin").message;
    expect(message).toEqual({ type: "sign.begin", hash: "SHA-256" });
    expect("algorithms" in message).toBe(false);
    expect("certificate" in message).toBe(false);
  });

  it("does not call prepare or send a digest before need_digest", async () => {
    const env = await loadSdk();
    const { prepare } = await beginSign(env);
    await flush();
    expect(prepare).not.toHaveBeenCalled();
    expect(digestsSent(env)).toEqual([]);
  });

  it("calls prepare with the certificate the app released and the chosen hash/algorithm", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    env.script.reply(
      id,
      needDigest(1, { algorithm: "RSASSA-PSS", certificate: wireCertificate() }),
    );
    await flush();
    expect(prepare).toHaveBeenCalledTimes(1);
    const [cert, context] = prepare.mock.calls[0] as [
      { fingerprint: string; der: Uint8Array; notBefore: Date },
      unknown,
    ];
    expect(cert.fingerprint).toBe(FINGERPRINT);
    expect(cert.der).toEqual(Uint8Array.of(1, 2, 3, 4));
    expect(cert.notBefore).toBeInstanceOf(Date);
    expect(context).toEqual({ hash: "SHA-256", algorithm: "RSASSA-PSS" });
  });

  it("sends the digest under the same request id with the same seq, Base64 encoded", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    prepare.mockReturnValue(bytes(32, 0x11));
    env.script.reply(id, needDigest(7));
    await flush();
    const sent = env.script.only("sign.digest");
    expect(sent.id).toBe(id);
    expect(sent.message).toEqual({ type: "sign.digest", seq: 7, digest: b64(bytes(32, 0x11)) });
  });

  it("awaits an async prepare", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    prepare.mockImplementation(async () => {
      await Promise.resolve();
      return bytes(32, 0x22);
    });
    env.script.reply(id, needDigest(1));
    await flush();
    expect(digestsSent(env)).toEqual([{ seq: 1, digest: b64(bytes(32, 0x22)) }]);
  });

  it("resolves with the converted result and decoded signature", async () => {
    const env = await loadSdk();
    const { s, id } = await beginSign(env);
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(
      id,
      signResult({ algorithm: "RSASSA-PSS", signature: b64(Uint8Array.of(1, 2, 3)) }),
    );
    await flush();
    const result = expectValue(s.outcome());
    expect(result.hash).toBe("SHA-256");
    expect(result.algorithm).toBe("RSASSA-PSS");
    expect(result.signature).toEqual(Uint8Array.of(1, 2, 3));
    expect(result.certificate.fingerprint).toBe(FINGERPRINT);
    expect(result.certificate.notAfter).toEqual(new Date(1792000000 * 1000));
  });

  it("re-runs prepare on a new need_digest and the last digest wins", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    prepare.mockReturnValueOnce(bytes(32, 1)).mockReturnValueOnce(bytes(32, 2));
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(
      id,
      needDigest(2, { certificate: wireCertificate({ fingerprint: "c".repeat(64) }) }),
    );
    await flush();
    expect(prepare).toHaveBeenCalledTimes(2);
    expect(prepare.mock.calls[1]?.[0]).toMatchObject({ fingerprint: "c".repeat(64) });
    expect(digestsSent(env).map((d) => d.seq)).toEqual([1, 2]);
  });

  it("ignores replies after the final one", async () => {
    const env = await loadSdk();
    const { prepare, id } = await beginSign(env);
    env.script.reply(id, signResult());
    await flush();
    env.script.reply(id, needDigest(9));
    await flush();
    expect(prepare).not.toHaveBeenCalled();
    expect(digestsSent(env)).toEqual([]);
  });

  it("sign.result with invalid Base64 rejects Internal", async () => {
    const env = await loadSdk();
    const { s, id } = await beginSign(env);
    env.script.reply(id, signResult({ signature: "not base64!" }));
    await flush();
    expectError(env, s.outcome(), "Internal");
  });

  it("a reply of the wrong type ends the request with an error", async () => {
    const env = await loadSdk();
    const { s, id } = await beginSign(env);
    env.script.reply(id, { type: "choose.result", certificates: [wireCertificate()] });
    await flush();
    expectError(env, s.outcome(), "Internal");
  });

  for (const code of ERROR_CODES) {
    it(`rejects ${code} from the app`, async () => {
      const env = await loadSdk();
      const { s, id } = await beginSign(env);
      env.script.reply(id, wireError(code, { details: { native: "CKR_DEVICE_ERROR" } }));
      await flush();
      const error = expectError(env, s.outcome(), code);
      expect(error.details).toEqual({ native: "CKR_DEVICE_ERROR" });
    });
  }

  it("rejects errors that arrive while prepare is running, and drops its digest", async () => {
    const env = await loadSdk();
    const { s, prepare, id } = await beginSign(env);
    let release!: (d: Uint8Array) => void;
    prepare.mockReturnValue(new Promise<Uint8Array>((r) => (release = r)));
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(id, wireError("Timeout"));
    await flush();
    release(bytes(32));
    await flush();
    expectError(env, s.outcome(), "Timeout");
    expect(digestsSent(env)).toEqual([]);
  });
});
