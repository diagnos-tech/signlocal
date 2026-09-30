/** Public types of the SDK. Wire types are generated; these are the friendly shapes. */

import type { CurveName, EidasType, KeyStorage } from "./generated";

/** A hash the digest was computed with. */
export type HashAlgorithm = "SHA-256" | "SHA-384" | "SHA-512";

/**
 * A signature scheme. RSASSA-PSS uses MGF1 with the same hash and a salt as long
 * as the digest. EdDSA is not offered: it signs messages, not hashes.
 */
export type SignatureAlgorithm = "ECDSA" | "RSASSA-PKCS1-v1_5" | "RSASSA-PSS";

/** The public key. */
export type KeyDescription = { type: "RSA"; bits: number } | { type: "EC"; curve: CurveName };

/** What the certificate declares; the app never claims legal qualification. */
export interface CertificateProfile {
  /** ICP-Brasil class ("A3", "S1", "T3"…), "ICP-Brasil" when unknown, absent when not ICP-Brasil. */
  readonly icpBrasil?: string;
  /** Present when the certificate has ETSI qcStatements. */
  readonly eidas?: {
    readonly qualified: boolean;
    readonly qscd: boolean;
    readonly types: readonly EidasType[];
  };
  readonly keyStorage: KeyStorage;
}

/** A certificate the person chose. */
export interface Certificate {
  /** DER bytes. */
  readonly der: Uint8Array;
  /** Issuers, nearest first, leaf excluded; best effort, may be empty. */
  readonly chain: readonly Uint8Array[];
  /** SHA-256 of `der`, 64 lowercase hex digits. */
  readonly fingerprint: string;
  /** Holder name as the app shows it. */
  readonly displayName: string;
  readonly issuerName: string;
  readonly notBefore: Date;
  readonly notAfter: Date;
  readonly key: KeyDescription;
  /** Algorithms this key can produce. */
  readonly algorithms: readonly SignatureAlgorithm[];
  readonly profile: CertificateProfile;
}

/** Passed to `prepare`, with the certificate. */
export interface PrepareContext {
  readonly hash: HashAlgorithm;
  /** The algorithm the signature will use (for CMS signatureAlgorithm / algorithm protection). */
  readonly algorithm: SignatureAlgorithm;
}

/** Options of {@link sign}. */
export interface SignOptions {
  readonly hash: HashAlgorithm;
  /** Acceptable algorithm(s), preferred first. Default: ECDSA for EC keys, PKCS#1 v1.5 for RSA. */
  readonly algorithm?: SignatureAlgorithm | readonly SignatureAlgorithm[];
  /** Preselect a certificate (from an earlier `certificates()`), by object or fingerprint. */
  readonly certificate?: Certificate | string;
  /**
   * Returns the digest to sign for `certificate` (exactly 32/48/64 bytes for
   * SHA-256/384/512). May run more than once if the person switches
   * certificate; only the last digest is signed. Throwing aborts the request.
   */
  readonly prepare: (
    certificate: Certificate,
    context: PrepareContext,
  ) => Uint8Array | ArrayBuffer | Promise<Uint8Array | ArrayBuffer>;
  /** Aborts the request (the window closes; the promise rejects with `Aborted`). */
  readonly signal?: AbortSignal;
}

/** The result of {@link sign}. */
export interface SignResult {
  readonly certificate: Certificate;
  readonly hash: HashAlgorithm;
  readonly algorithm: SignatureAlgorithm;
  /** RSA: the signature block. ECDSA: raw r‖s (IEEE P1363), each half padded to the curve size. */
  readonly signature: Uint8Array;
}

/** Options of {@link certificates}. */
export interface CertificateOptions {
  /** Only certificates whose key can produce one of these. */
  readonly algorithm?: SignatureAlgorithm | readonly SignatureAlgorithm[];
  readonly signal?: AbortSignal;
}

/** What {@link status} reports. Never opens a window. */
export interface Status {
  readonly extension: { readonly installed: boolean; readonly version?: string };
  readonly app: {
    readonly installed: boolean;
    readonly version?: string;
    /** Older than the extension's minimum: signing will fail with AppOutdated. */
    readonly outdated: boolean;
  };
  /** The person ticked "Remember this site": `certificates()` answers without a window. */
  readonly remembered: boolean;
  /** Extension and app present and compatible. */
  readonly ready: boolean;
}

/** The verification code the app shows next to its Sign button. */
export interface VerificationCode {
  /** First 8 digest bytes, uppercase hex, 4 groups of 4: "7F3A 9C21 E0B4 55D8". */
  readonly text: string;
  /** Identicon color 0..7 (palette in docs/ux.md §11.1). */
  readonly colorIndex: number;
  /** 25 cells, row-major, true = lit; columns 3–4 mirror 1–0. */
  readonly cells: readonly boolean[];
}
