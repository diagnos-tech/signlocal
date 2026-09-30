import { describe, expect, it } from "vitest";
import { loadSdk, useFakeEnvironment } from "./helpers/env";
import { expectValue } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { bytes, needDigest, signResult, wireCertificate } from "./helpers/fixtures";
import { beginSign } from "./helpers/sign";

useFakeEnvironment();

describe("sign() result.digest", () => {
  it("is the digest prepare returned, as a Uint8Array copy", async () => {
    const env = await loadSdk();
    const returned = bytes(32, 7);
    const { s, id, prepare } = await beginSign(env);
    prepare.mockReturnValueOnce(returned.buffer as ArrayBuffer);
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(id, signResult());
    await flush();
    const { digest } = expectValue(s.outcome());
    expect(digest).toBeInstanceOf(Uint8Array);
    expect(digest).toEqual(bytes(32, 7));
    returned.fill(0);
    expect(digest).toEqual(bytes(32, 7));
  });

  it("is the last digest sent when the person switched certificate", async () => {
    const env = await loadSdk();
    const { s, id, prepare } = await beginSign(env);
    prepare.mockReturnValueOnce(bytes(32, 1)).mockReturnValueOnce(bytes(32, 2));
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(
      id,
      needDigest(2, { certificate: wireCertificate({ fingerprint: "c".repeat(64) }) }),
    );
    await flush();
    env.script.reply(id, signResult());
    await flush();
    expect(expectValue(s.outcome()).digest).toEqual(bytes(32, 2));
  });

  it("only the view of a Uint8Array is sent and returned", async () => {
    const env = await loadSdk();
    const { s, id, prepare } = await beginSign(env);
    const backing = bytes(40, 9);
    backing.fill(3, 0, 4);
    prepare.mockReturnValueOnce(backing.subarray(4, 36));
    env.script.reply(id, needDigest(1));
    await flush();
    env.script.reply(id, signResult());
    await flush();
    expect(expectValue(s.outcome()).digest).toEqual(bytes(32, 9));
  });
});
