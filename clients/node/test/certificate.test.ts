import { describe, expect, it } from "vitest";
import { Signlocal } from "../src/index";
import { CERTIFICATE, fake, helloRule, type Rule, send } from "./support";

const withBytes = { ...CERTIFICATE, der: "MIIB", chain: ["AQID", "BAUG"] };

async function connected(rules: Rule[]) {
  const app = fake({ rules: [helloRule(), ...rules] });
  return { app, signlocal: await Signlocal.connect({ executable: app.executable }) };
}

describe("certificates are decoded at the boundary", () => {
  it("certificates() returns der and chain as bytes, validity as Date, other fields as sent", async () => {
    const { signlocal } = await connected([
      { on: "choose", do: [send({ type: "choose.result", certificates: [withBytes] })] },
    ]);
    const [certificate] = await signlocal.certificates();
    await signlocal.close();
    expect(certificate?.der).toBeInstanceOf(Uint8Array);
    expect([...(certificate?.der ?? [])]).toEqual([0x30, 0x82, 0x01]);
    expect(certificate?.chain.map((c) => [...c])).toEqual([
      [1, 2, 3],
      [4, 5, 6],
    ]);
    expect(certificate?.notBefore).toEqual(new Date(withBytes.notBefore * 1000));
    expect(certificate?.notAfter).toEqual(new Date(withBytes.notAfter * 1000));
    const { der: _der, chain: _chain, notBefore: _nb, notAfter: _na, ...rest } = withBytes;
    expect(certificate).toMatchObject(rest);
  });

  it("prepare and the sign result get decoded certificates", async () => {
    const { signlocal } = await connected([
      {
        on: "sign.begin",
        do: [
          send({
            type: "sign.need_digest",
            seq: 1,
            hash: "SHA-256",
            algorithm: "ECDSA",
            certificate: withBytes,
          }),
        ],
      },
      {
        on: "sign.digest",
        do: [
          send({
            type: "sign.result",
            hash: "SHA-256",
            algorithm: "ECDSA",
            certificate: withBytes,
            signature: "AQID",
          }),
        ],
      },
    ]);
    const seen: Uint8Array[] = [];
    const signed = await signlocal.sign({
      hash: "SHA-256",
      prepare: (certificate) => {
        seen.push(certificate.der);
        return new Uint8Array(32);
      },
    });
    await signlocal.close();
    expect(seen.map((der) => [...der])).toEqual([[0x30, 0x82, 0x01]]);
    expect(signed.certificate.chain.map((c) => [...c])).toEqual([
      [1, 2, 3],
      [4, 5, 6],
    ]);
  });

  it.each([
    ["non-canonical der", { ...CERTIFICATE, der: "AQI" }],
    ["a chain entry that is not Base64", { ...CERTIFICATE, chain: ["**"] }],
    ["a missing chain", { ...CERTIFICATE, chain: undefined }],
    ["a validity date that is not a number", { ...CERTIFICATE, notAfter: "soon" }],
  ])("certificates() rejects Internal on %s", async (_name, certificate) => {
    const { signlocal } = await connected([
      { on: "choose", do: [send({ type: "choose.result", certificates: [certificate] })] },
    ]);
    await expect(signlocal.certificates()).rejects.toMatchObject({ code: "Internal" });
    await signlocal.close();
  });

  it("cancels the signature when the digest request carries a garbled certificate", async () => {
    const { app, signlocal } = await connected([
      {
        on: "sign.begin",
        do: [
          send({
            type: "sign.need_digest",
            seq: 1,
            hash: "SHA-256",
            algorithm: "ECDSA",
            certificate: { ...CERTIFICATE, der: "%%%%" },
          }),
        ],
      },
    ]);
    let called = false;
    const signing = signlocal.sign({
      hash: "SHA-256",
      prepare: () => {
        called = true;
        return new Uint8Array(32);
      },
    });
    await expect(signing).rejects.toMatchObject({ code: "Internal" });
    await signlocal.close();
    expect(called).toBe(false);
    expect(app.received().at(-1)).toMatchObject({ type: "cancel", id: "n2" });
  });
});
