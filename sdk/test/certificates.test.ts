import { describe, expect, it } from "vitest";
import { loadSdk, settle, useFakeEnvironment } from "./helpers/env";
import { expectError, expectValue } from "./helpers/expect";
import { flush } from "./helpers/fake-script";
import { ERROR_CODES, FINGERPRINT, wireCertificate, wireError } from "./helpers/fixtures";

useFakeEnvironment();

const choose = (...certs: unknown[]) => ({ type: "choose.result", certificates: certs });

describe("certificates()", () => {
  it("asks with a bare choose request: nothing that lists the machine (D2)", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [choose(wireCertificate())];
    settle(env.sdk.certificates());
    await flush();
    expect(env.script.only("choose").message).toEqual({ type: "choose" });
    expect(env.script.requests.map((r) => r.message.type)).toEqual(["choose"]);
  });

  it("returns exactly the certificate the person chose, converted", async () => {
    const env = await loadSdk();
    env.script.autoReply = () => [
      choose(
        wireCertificate({
          profile: {
            keyStorage: "software",
            eidas: { qualified: true, qscd: false, types: ["esign"] },
          },
        }),
      ),
    ];
    const s = settle(env.sdk.certificates());
    await flush();
    const list = expectValue(s.outcome());
    expect(list).toHaveLength(1);
    const cert = list[0];
    expect(cert?.der).toEqual(Uint8Array.of(1, 2, 3, 4));
    expect(cert?.chain).toEqual([Uint8Array.of(9, 8, 7)]);
    expect(cert?.fingerprint).toBe(FINGERPRINT);
    expect(cert?.displayName).toBe("Ana Beatriz Souza");
    expect(cert?.notBefore).toEqual(new Date(1741000000 * 1000));
    expect(cert?.notAfter).toEqual(new Date(1792000000 * 1000));
    expect(cert?.key).toEqual({ type: "RSA", bits: 2048 });
    expect(cert?.profile.eidas).toEqual({ qualified: true, qscd: false, types: ["esign"] });
  });

  it("passes a remembered caller's certificates through in the order received", async () => {
    const env = await loadSdk();
    const a = wireCertificate({ fingerprint: "a".repeat(64) });
    const b = wireCertificate({ fingerprint: "b".repeat(64) });
    env.script.autoReply = () => [choose(a, b)];
    const s = settle(env.sdk.certificates());
    await flush();
    expect(expectValue(s.outcome()).map((c) => c.fingerprint)).toEqual([
      "a".repeat(64),
      "b".repeat(64),
    ]);
  });

  it("an empty result is NoCertificates, never an empty list", async () => {
    // The app reports an empty choice as the NoCertificates error; an empty list is refused too.
    const env = await loadSdk();
    env.script.autoReply = () => [choose()];
    const s = settle(env.sdk.certificates());
    await flush();
    expectError(env, s.outcome(), "NoCertificates");
  });

  describe("algorithm filter", () => {
    const cases: [string, unknown, unknown][] = [
      ["single", "ECDSA", { algorithms: ["ECDSA"] }],
      ["array keeps order", ["RSASSA-PSS", "ECDSA"], { algorithms: ["RSASSA-PSS", "ECDSA"] }],
      [
        "deduplicates keeping first occurrence",
        ["ECDSA", "RSASSA-PSS", "ECDSA"],
        { algorithms: ["ECDSA", "RSASSA-PSS"] },
      ],
    ];
    for (const [name, input, filter] of cases) {
      it(name, async () => {
        const env = await loadSdk();
        settle(env.sdk.certificates({ algorithm: input as never }));
        await flush();
        expect(env.script.only("choose").message).toEqual({ type: "choose", filter });
      });
    }

    it("omitted option sends no filter key", async () => {
      const env = await loadSdk();
      settle(env.sdk.certificates({}));
      await flush();
      expect("filter" in env.script.only("choose").message).toBe(false);
    });

    for (const bad of ["EdDSA", "ecdsa", "", 5, null, [], ["ECDSA", "RS256"]]) {
      it(`rejects InvalidRequest for ${JSON.stringify(bad)} and sends nothing`, async () => {
        // An empty array is invalid too: the protocol's filter is non-empty (SPEC §5).
        const env = await loadSdk();
        const s = settle(env.sdk.certificates({ algorithm: bad as never }));
        await flush();
        expectError(env, s.outcome(), "InvalidRequest");
        expect(env.win.posted).toEqual([]);
      });
    }
  });

  describe("errors", () => {
    for (const code of ERROR_CODES) {
      it(`maps error ${code} to WebSignError(${code}) with message and details`, async () => {
        const env = await loadSdk();
        const details = { installed: "1.0.0", required: "1.4.0", native: "0x5" };
        env.script.autoReply = () => [wireError(code, { details })];
        const s = settle(env.sdk.certificates());
        await flush();
        const error = expectError(env, s.outcome(), code);
        expect(error.message).toBe(`test ${code}`);
        expect(error.details).toEqual(details);
        expect(error).toBeInstanceOf(Error);
      });
    }

    it("an unknown error code becomes Internal", async () => {
      const env = await loadSdk();
      env.script.autoReply = () => [wireError("Nonsense")];
      const s = settle(env.sdk.certificates());
      await flush();
      expectError(env, s.outcome(), "Internal");
    });

    it("a reply of the wrong type for the request is refused", async () => {
      // choose accepts only choose.result or error; anything else is the extension's bug.
      const env = await loadSdk();
      env.script.autoReply = () => [{ type: "sign.result" }];
      const s = settle(env.sdk.certificates());
      await flush();
      expectError(env, s.outcome(), "Internal");
    });
  });

  it("abort posts cancel with the same id and rejects Aborted without waiting", async () => {
    const env = await loadSdk();
    const ctl = new AbortController();
    const s = settle(env.sdk.certificates({ signal: ctl.signal }));
    await flush();
    const id = env.script.only("choose").id;
    ctl.abort();
    await flush();
    expect(env.script.only("cancel")).toMatchObject({ id, message: { type: "cancel" } });
    expectError(env, s.outcome(), "Aborted");
  });

  it("an already aborted signal rejects Aborted and sends nothing", async () => {
    const env = await loadSdk();
    const s = settle(env.sdk.certificates({ signal: AbortSignal.abort() }));
    await flush();
    expectError(env, s.outcome(), "Aborted");
    expect(env.script.requests).toEqual([]);
  });
});
