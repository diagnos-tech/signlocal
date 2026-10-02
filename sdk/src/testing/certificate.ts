/**
 * Fake certificates: real, parseable X.509 v3 DER, self-issued with the
 * public test keys, so a site under test can hand them to its CMS/PDF code and
 * verify the signatures it gets back.
 */

import { toBase64 } from "../convert.js";
import type { EidasType, KeyStorage, Certificate as WireCertificate } from "../generated/index.js";
import type { HashAlgorithm, SignatureAlgorithm } from "../types.js";
import { fromBase64Url, toBigInt, toBytes } from "./bigint.js";
import * as der from "./der.js";
import { TEST_EC_KEY, TEST_RSA_KEY } from "./keys.js";
import { signP256 } from "./p256.js";
import { signRsa } from "./rsa.js";

/**
 * One certificate of the fake. Everything is optional; the defaults give a
 * valid EC P-256 certificate of a fictional "Test Signer".
 *
 * @example
 * installFakeWebSign({ certificates: [{ key: "RSA", icpBrasil: "A3", displayName: "Maria Test" }] });
 */
export interface FakeCertificateOptions {
  /** Default "EC" (P-256, ECDSA); "RSA" is 2048 bits with PKCS#1 v1.5 and PSS. */
  readonly key?: "EC" | "RSA";
  /** Holder name; default "Test Signer EC (fake)" or "Test Signer RSA (fake)". */
  readonly displayName?: string;
  /** Default "WebeSign Testing CA (fake)". */
  readonly issuerName?: string;
  /** Default 2025-01-01. */
  readonly notBefore?: Date;
  /** Default 2035-01-01. Put it in the past to test `CertificateNotValid`. */
  readonly notAfter?: Date;
  /** Reported ICP-Brasil class (profile only; the DER has no policy). */
  readonly icpBrasil?: string;
  /** Reported eIDAS profile (profile only). */
  readonly eidas?: {
    readonly qualified: boolean;
    readonly qscd: boolean;
    readonly types: readonly EidasType[];
  };
  /** Default "hardware", like a token. */
  readonly keyStorage?: KeyStorage;
}

/** A certificate with the key that signs for it. */
export interface FakeCredential {
  readonly wire: WireCertificate;
  /** The public key, for `verify()`. */
  readonly jwk: JsonWebKey;
  sign(algorithm: SignatureAlgorithm, hash: HashAlgorithm, digest: Uint8Array): Promise<Uint8Array>;
}

const ECDSA_WITH_SHA256 = "1.2.840.10045.4.3.2";
const SHA256_WITH_RSA = "1.2.840.113549.1.1.11";

const name = (commonName: string) =>
  der.sequence(
    der.set(der.sequence(der.oid("2.5.4.10"), der.utf8String("WebeSign testing (fake)"))),
    der.set(der.sequence(der.oid("2.5.4.3"), der.utf8String(commonName))),
  );

async function sha256(data: Uint8Array): Promise<Uint8Array> {
  return new Uint8Array(await crypto.subtle.digest("SHA-256", data as BufferSource));
}

function publicJwk({ kty = "", crv = "", x = "", y = "", n = "", e = "" }: JsonWebKey): JsonWebKey {
  return kty === "EC" ? { kty, crv, x, y } : { kty, n, e };
}

async function spkiOf(jwk: JsonWebKey): Promise<Uint8Array> {
  const algorithm =
    jwk.kty === "EC"
      ? { name: "ECDSA", namedCurve: "P-256" }
      : { name: "RSASSA-PKCS1-v1_5", hash: "SHA-256" };
  const key = await crypto.subtle.importKey("jwk", jwk, algorithm, true, ["verify"]);
  return new Uint8Array(await crypto.subtle.exportKey("spki", key));
}

/** Raw r‖s → DER Ecdsa-Sig-Value, as X.509 stores ECDSA signatures. */
const ecdsaDer = (raw: Uint8Array) =>
  der.sequence(der.unsignedInteger(raw.subarray(0, 32)), der.unsignedInteger(raw.subarray(32)));

/** Builds certificate number `index` (it seeds the serial number). */
export async function createCredential(
  options: FakeCertificateOptions,
  index: number,
): Promise<FakeCredential> {
  const isEc = options.key !== "RSA";
  const privateJwk = isEc ? TEST_EC_KEY : TEST_RSA_KEY;
  const sign = (algorithm: SignatureAlgorithm, hash: HashAlgorithm, digest: Uint8Array) =>
    isEc
      ? signP256(fromBase64Url(privateJwk.d ?? ""), digest)
      : signRsa(privateJwk, algorithm, hash, digest);

  const displayName = options.displayName ?? `Test Signer ${isEc ? "EC" : "RSA"} (fake)`;
  const issuerName = options.issuerName ?? "WebeSign Testing CA (fake)";
  const notBefore = options.notBefore ?? new Date(Date.UTC(2025, 0, 1));
  const notAfter = options.notAfter ?? new Date(Date.UTC(2035, 0, 1));
  const seed = await sha256(new TextEncoder().encode(`${index}:${displayName}`));
  const serial = toBytes(toBigInt(seed.subarray(0, 16)) >> 1n, 16);
  const signatureAlgorithm = isEc
    ? der.sequence(der.oid(ECDSA_WITH_SHA256))
    : der.sequence(der.oid(SHA256_WITH_RSA), der.nullValue());
  const keyUsage = der.sequence(
    der.oid("2.5.29.15"),
    der.boolean(true),
    der.octetString(der.bitString(Uint8Array.of(0xc0), 6)), // digitalSignature, nonRepudiation
  );
  const tbs = der.sequence(
    der.explicit(0, der.unsignedInteger(Uint8Array.of(2))), // v3
    der.unsignedInteger(serial),
    signatureAlgorithm,
    name(issuerName),
    der.sequence(der.time(notBefore), der.time(notAfter)),
    name(displayName),
    await spkiOf(publicJwk(privateJwk)),
    der.explicit(3, der.sequence(keyUsage)),
  );
  const tbsSignature = await sign("RSASSA-PKCS1-v1_5", "SHA-256", await sha256(tbs));
  const certificate = der.sequence(
    tbs,
    signatureAlgorithm,
    der.bitString(isEc ? ecdsaDer(tbsSignature) : tbsSignature),
  );
  const fingerprint = Array.from(await sha256(certificate), (b) =>
    b.toString(16).padStart(2, "0"),
  ).join("");

  const { icpBrasil, eidas } = options;
  return {
    jwk: publicJwk(privateJwk),
    sign,
    wire: {
      der: toBase64(certificate),
      chain: [],
      fingerprint,
      displayName,
      issuerName,
      notBefore: Math.floor(notBefore.getTime() / 1000),
      notAfter: Math.floor(notAfter.getTime() / 1000),
      key: isEc ? { type: "EC", curve: "P-256" } : { type: "RSA", bits: 2048 },
      algorithms: isEc ? ["ECDSA"] : ["RSASSA-PKCS1-v1_5", "RSASSA-PSS"],
      profile: {
        ...(icpBrasil !== undefined && { icpBrasil }),
        ...(eidas !== undefined && { eidas: { ...eidas, types: [...eidas.types] } }),
        keyStorage: options.keyStorage ?? "hardware",
      },
    },
  };
}
