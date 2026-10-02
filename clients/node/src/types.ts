import type {
  CertificateProfile,
  HashName,
  KeyDescription,
  SignatureAlgorithmName,
} from "./generated/index.js";

/**
 * A hash the digest was computed with: `"SHA-256"`, `"SHA-384"` or `"SHA-512"`.
 * Same union as `@websign/sdk`.
 *
 * @example
 * ```ts
 * const hash: HashAlgorithm = "SHA-256";
 * ```
 */
export type HashAlgorithm = HashName;

/**
 * A signature scheme. RSASSA-PSS uses MGF1 with the same hash and a salt as
 * long as the digest. Same union as `@websign/sdk`.
 *
 * @example
 * ```ts
 * const algorithm: SignatureAlgorithm = "RSASSA-PSS"; // or "ECDSA", "RSASSA-PKCS1-v1_5"
 * ```
 */
export type SignatureAlgorithm = SignatureAlgorithmName;

/**
 * How to start the app. Every field is optional.
 *
 * @example
 * ```ts
 * await WebSign.connect({ executable: "/opt/websign/websign", clientName: "my-app" });
 * ```
 */
export interface ConnectOptions {
  /** Path of `websign`; default {@link findExecutable}. */
  readonly executable?: string;
  /**
   * Arguments placed before `connect`, for starting the app through an
   * interpreter or wrapper (the `@websign/desktop/testing` fake uses it to
   * run under `node`). The installed app never needs it.
   */
  readonly executableArgs?: readonly string[];
  /** Sent in `hello` (logs only). */
  readonly clientName?: string;
  /** Sent in `hello` (logs only); default this library's version. */
  readonly clientVersion?: string;
}

/**
 * A certificate the person chose, shaped like `@websign/sdk`'s so code that
 * builds signature formats serves both: `der` and `chain` are bytes (what
 * CMS/PAdES code consumes) and the validity bounds are `Date`s.
 *
 * @example
 * ```ts
 * const [certificate] = await websign.certificates();
 * const expires = certificate.notAfter.toISOString();
 * const leaf = certificate.der; // Uint8Array, feed it to your CMS builder
 * ```
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
  /** Issuer name as the app shows it. */
  readonly issuerName: string;
  readonly notBefore: Date;
  readonly notAfter: Date;
  readonly key: KeyDescription;
  /** Algorithms this key can produce. */
  readonly algorithms: readonly SignatureAlgorithm[];
  readonly profile: CertificateProfile;
}

/**
 * Passed to `prepare`, with the certificate.
 *
 * @example
 * ```ts
 * prepare: (certificate, { hash, algorithm }) => digestOfSignedAttributes(certificate.der, hash, algorithm)
 * ```
 */
export interface PrepareContext {
  readonly hash: HashAlgorithm;
  /** The algorithm the signature will use (for CMS signatureAlgorithm / algorithm protection). */
  readonly algorithm: SignatureAlgorithm;
}

/**
 * What to sign.
 *
 * @example
 * ```ts
 * const options: SignOptions = {
 *   hash: "SHA-256",
 *   algorithm: "ECDSA",
 *   prepare: () => new Uint8Array(32), // your real digest, 32 bytes for SHA-256
 * };
 * ```
 */
export interface SignOptions {
  /** Hash `prepare`'s digest was computed with. */
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

/**
 * Options of `certificates()`.
 *
 * @example
 * ```ts
 * await websign.certificates({ algorithm: ["ECDSA", "RSASSA-PSS"], signal: AbortSignal.timeout(60_000) });
 * ```
 */
export interface CertificateOptions {
  /** Only certificates whose key can produce one of these. */
  readonly algorithm?: SignatureAlgorithm | readonly SignatureAlgorithm[];
  /** Aborting sends `cancel` and rejects with `Aborted`. */
  readonly signal?: AbortSignal;
}

/**
 * The signature and what it was made with (bytes decoded).
 *
 * @example
 * ```ts
 * const result = await websign.sign({ hash: "SHA-256", prepare });
 * await writeFile("contract.sig", result.signature);
 * console.log(result.algorithm, result.certificate.displayName);
 * ```
 */
export interface SignResult {
  /** The certificate that signed. */
  readonly certificate: Certificate;
  readonly hash: HashAlgorithm;
  /** The scheme the app used (one of those you accepted). */
  readonly algorithm: SignatureAlgorithm;
  /** RSA: the signature block. ECDSA: raw `r || s` (IEEE P1363). */
  readonly signature: Uint8Array;
}
