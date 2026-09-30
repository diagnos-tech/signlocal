import type { Certificate as WireCertificate } from "../../src/generated";

export const FINGERPRINT = "256adb9a".repeat(8);

export const DIGEST_LENGTH = { "SHA-256": 32, "SHA-384": 48, "SHA-512": 64 } as const;

export function bytes(length: number, fill = 0xab): Uint8Array {
  return new Uint8Array(length).fill(fill);
}

export function b64(data: Uint8Array): string {
  let text = "";
  for (const byte of data) text += String.fromCharCode(byte);
  return btoa(text);
}

export function wireCertificate(overrides: Partial<WireCertificate> = {}): WireCertificate {
  return {
    der: b64(Uint8Array.of(1, 2, 3, 4)),
    chain: [b64(Uint8Array.of(9, 8, 7))],
    fingerprint: FINGERPRINT,
    displayName: "Ana Beatriz Souza",
    issuerName: "AC SOLUTI Multipla v5",
    notBefore: 1741000000,
    notAfter: 1792000000,
    key: { type: "RSA", bits: 2048 },
    algorithms: ["RSASSA-PKCS1-v1_5", "RSASSA-PSS"],
    profile: { icpBrasil: "A3", keyStorage: "hardware" },
    ...overrides,
  };
}

export function needDigest(
  seq: number,
  overrides: Record<string, unknown> = {},
): Record<string, unknown> {
  return {
    type: "sign.need_digest",
    seq,
    hash: "SHA-256",
    algorithm: "RSASSA-PKCS1-v1_5",
    certificate: wireCertificate(),
    ...overrides,
  };
}

export function signResult(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    type: "sign.result",
    hash: "SHA-256",
    algorithm: "RSASSA-PKCS1-v1_5",
    certificate: wireCertificate(),
    signature: b64(bytes(256, 0x5a)),
    ...overrides,
  };
}

export function wireError(
  code: string,
  extra: Record<string, unknown> = {},
): Record<string, unknown> {
  return { type: "error", code, message: `test ${code}`, ...extra };
}

export function pageStatus(overrides: Record<string, unknown> = {}): Record<string, unknown> {
  return {
    type: "status",
    extension: { version: "1.4.2", browser: "chrome" },
    app: {
      version: "1.4.0",
      protocols: { min: 1, max: 1 },
      os: "linux",
      arch: "x86_64",
      channel: "direct",
    },
    appOutdated: false,
    remembered: false,
    ...overrides,
  };
}

export const ERROR_CODES = [
  "ExtensionMissing",
  "AppMissing",
  "AppOutdated",
  "ExtensionOutdated",
  "ClientOutdated",
  "InsecureOrigin",
  "Aborted",
  "UserCancelled",
  "Timeout",
  "NoCertificates",
  "CertificateUnavailable",
  "CertificateNotValid",
  "InvalidRequest",
  "UnsupportedAlgorithm",
  "PinIncorrect",
  "PinLocked",
  "TokenRemoved",
  "DriverFailure",
  "Busy",
  "Internal",
] as const;
