import type {
  CertificateProfile,
  HashName,
  KeyDescription,
  SignatureAlgorithmName,
} from "./generated/index.js";

/** A hash the digest was computed with. Same union as `@websign/sdk`. */
export type HashAlgorithm = HashName;

/**
 * A signature scheme. RSASSA-PSS uses MGF1 with the same hash and a salt as
 * long as the digest. Same union as `@websign/sdk`.
 */
export type SignatureAlgorithm = SignatureAlgorithmName;

/** How to start the app. */
export interface ConnectOptions {
  /** Path of `websign`; default {@link findExecutable}. */
  readonly executable?: string;
  /** Sent in `hello` (logs only). */
  readonly clientName?: string;
  readonly clientVersion?: string;
}

/**
 * A certificate the person chose, shaped like `@websign/sdk`'s so code that
 * builds signature formats serves both: `der` and `chain` are bytes (what
 * CMS/PAdES code consumes) and the validity bounds are `Date`s.
 */
export interface Certificate {
  /** The certificate, DER. */
  readonly der: Uint8Array;
  /** Issuer certificates (DER), leaf excluded, nearest first; best effort, may be empty. */
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

/** What to sign. */
export interface SignOptions {
  readonly hash: HashAlgorithm;
  /** Acceptable algorithm(s), preferred first. Default: ECDSA for EC keys, PKCS#1 v1.5 for RSA. */
  readonly algorithm?: SignatureAlgorithm | readonly SignatureAlgorithm[];
  /** Preselect a certificate (from an earlier `certificates()`), by object or fingerprint. */
  readonly certificate?: Certificate | string;
  /**
   * Digest for `certificate` (exactly 32/48/64 bytes for SHA-256/384/512);
   * may run more than once, because the person can switch certificate in the
   * window (the digest of a PAdES/CAdES document covers the certificate, so
   * each switch needs a new one). Throwing aborts the signature and rejects
   * with `Aborted`, the error as `cause`.
   */
  readonly prepare: (
    certificate: Certificate,
    context: PrepareContext,
  ) => Uint8Array | ArrayBuffer | Promise<Uint8Array | ArrayBuffer>;
  /** Aborting sends `cancel` and rejects with `Aborted`. */
  readonly signal?: AbortSignal;
}

/** Options of `certificates()`. */
export interface CertificateOptions {
  /** Only certificates whose key can produce one of these. */
  readonly algorithm?: SignatureAlgorithm | readonly SignatureAlgorithm[];
  /** Aborting sends `cancel` and rejects with `Aborted`. */
  readonly signal?: AbortSignal;
}

/** The signature and what it was made with (bytes decoded). */
export interface SignResult {
  readonly certificate: Certificate;
  readonly hash: HashAlgorithm;
  readonly algorithm: SignatureAlgorithm;
  /** RSA: the signature block. ECDSA: raw `r || s` (IEEE P1363). */
  readonly signature: Uint8Array;
}
