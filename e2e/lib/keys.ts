/**
 * The software keys of this run, known by their certificates: which key a
 * scenario preselects (by fingerprint), what it can sign with, and the
 * holder names the log must never contain.
 */

import { createHash, X509Certificate } from "node:crypto";
import { existsSync, readdirSync, readFileSync } from "node:fs";
import { join } from "node:path";

import type { KeySource } from "./environment.ts";

/** One software key, by its certificate. */
export interface TestKey {
  /** Folder or file name (`rsa-2048`, `ec-p256`…). */
  readonly name: string;
  readonly certificate: X509Certificate;
  /** SHA-256 of the DER, lowercase hex: the SDK's fingerprint. */
  readonly fingerprint: string;
  /** Common name of the holder. */
  readonly holder: string;
  readonly type: "RSA" | "EC";
  /** EC curve (`P-256`…), absent for RSA. */
  readonly curve?: string;
}

const CURVES: Record<string, string> = {
  prime256v1: "P-256",
  secp384r1: "P-384",
  secp521r1: "P-521",
};

/** Every key the source holds whose certificate can be read. */
export function testKeys(source: KeySource): TestKey[] {
  const files =
    source.kind === "softhsm"
      ? readdirSync(join(source.token.dir, "keys"))
          .sort()
          .map((name) => ({ name, path: join(source.token.dir, "keys", name, "cert.der") }))
      : readdirSync(source.certificates)
          .filter((file) => /\.(cer|crt|der|pem)$/i.test(file))
          .sort()
          .map((file) => ({
            name: file.replace(/\.[^.]+$/, ""),
            path: join(source.certificates, file),
          }));
  return files.filter(({ path }) => existsSync(path)).map(({ name, path }) => describe(name, path));
}

function describe(name: string, path: string): TestKey {
  const certificate = new X509Certificate(readFileSync(path));
  const key = certificate.publicKey;
  const curve = key.asymmetricKeyDetails?.namedCurve;
  return {
    name,
    certificate,
    fingerprint: createHash("sha256").update(certificate.raw).digest("hex"),
    holder: /CN=([^\n,]+)/.exec(certificate.subject)?.[1] ?? name,
    type: key.asymmetricKeyType === "ec" ? "EC" : "RSA",
    ...(curve === undefined ? {} : { curve: CURVES[curve] ?? curve }),
  };
}

/** The first RSA key, and the first key on each NIST curve. */
export function signingKeys(keys: readonly TestKey[]): TestKey[] {
  const pick = (test: (key: TestKey) => boolean) => keys.filter(test).slice(0, 1);
  return [
    ...pick((k) => k.type === "RSA"),
    ...pick((k) => k.curve === "P-256"),
    ...pick((k) => k.curve === "P-384"),
    ...pick((k) => k.curve === "P-521"),
  ];
}
