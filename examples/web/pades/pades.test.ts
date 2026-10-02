/**
 * Runs the PAdES flow against the SDK's fake and checks the result with
 * OpenSSL, independently of the code that built it: the CMS signature
 * verifies over the PDF's byte ranges, messageDigest and the
 * signing-certificate-v2 attribute included (`openssl cms -verify`).
 * Run with `bun test pades`; skipped when openssl is not installed.
 */

import { afterAll, beforeAll, describe, expect, it } from "bun:test";
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

// A browser-like window: the SDK and the fake talk through its message events.
class TestWindow extends EventTarget {
  readonly location = { origin: "http://localhost:5173", protocol: "http:", hostname: "localhost" };
  postMessage(data: unknown): void {
    const event = Object.assign(new Event("message"), {
      data: structuredClone(data),
      origin: this.location.origin,
      source: this,
    });
    setTimeout(() => this.dispatchEvent(event), 0);
  }
}
const win = new TestWindow();
Object.assign(globalThis, { window: win, location: win.location });

const { installFakeWebSign } = await import("@websign/sdk/testing");
const { samplePdf } = await import("./pdf");
const { signPdf } = await import("./sign-pdf");

const openssl = Bun.which("openssl");
const dir = mkdtempSync(join(tmpdir(), "websign-pades-"));
afterAll(() => rmSync(dir, { recursive: true, force: true }));
beforeAll(() => {
  console.warn = () => {}; // the fake's "never ship me" warning
});

/** The CMS and the covered bytes, read back from the PDF like a validator does. */
function extract(pdf: Uint8Array) {
  const text = new TextDecoder("latin1").decode(pdf);
  const match = /\/ByteRange \[(\d+) (\d+) (\d+) (\d+)\]/.exec(text);
  if (!match) throw new Error("no ByteRange");
  const [a, b, c, d] = match.slice(1).map(Number) as [number, number, number, number];
  const hex = text.slice(b + 1, c - 1).replace(/(00)+$/, "");
  const cms = Uint8Array.from(hex.match(/../g) ?? [], (h) => Number.parseInt(h, 16));
  const content = new Uint8Array([...pdf.subarray(a, a + b), ...pdf.subarray(c, c + d)]);
  return { cms, content, covered: a + b + d === pdf.length - (c - b) };
}

function verifyWithOpenSsl(pdf: Uint8Array, name: string) {
  const { cms, content, covered } = extract(pdf);
  expect(covered).toBe(true);
  writeFileSync(join(dir, `${name}.der`), cms);
  writeFileSync(join(dir, `${name}.bin`), content);
  const run = Bun.spawnSync([
    openssl ?? "openssl",
    "cms",
    "-verify",
    "-binary",
    "-inform",
    "DER",
    "-in",
    join(dir, `${name}.der`),
    "-content",
    join(dir, `${name}.bin`),
    "-noverify",
    "-out",
    "/dev/null",
  ]);
  expect(run.stderr.toString()).toContain("Verification successful");
}

describe.skipIf(!openssl)("PAdES with the fake", () => {
  it("ECDSA P-256: OpenSSL verifies the embedded CMS", async () => {
    const fake = await installFakeWebSign();
    const signed = await signPdf(await samplePdf());
    expect(signed.certificate.key.type).toBe("EC");
    verifyWithOpenSsl(signed.pdf, "ecdsa");
    fake.uninstall();
  });

  it("RSA PKCS#1 v1.5: OpenSSL verifies the embedded CMS", async () => {
    const fake = await installFakeWebSign({ certificates: [{ key: "RSA" }] });
    const signed = await signPdf(await samplePdf());
    expect(signed.certificate.key.type).toBe("RSA");
    verifyWithOpenSsl(signed.pdf, "rsa");
    fake.uninstall();
  });

  it("the person switching certificate still yields a valid signature for the second one", async () => {
    const fake = await installFakeWebSign();
    fake.switchDuringNextSign(1);
    const signed = await signPdf(await samplePdf());
    expect(signed.certificate.fingerprint).toBe(fake.certificates[1]?.fingerprint ?? "");
    verifyWithOpenSsl(signed.pdf, "switched");
    fake.uninstall();
  });
});
