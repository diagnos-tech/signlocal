import type {
  HashName,
  SignatureAlgorithmName,
  Certificate as WireCertificate,
} from "./generated/index.js";

/** How to start the app. */
export interface ConnectOptions {
  /** Path of `websign`; default {@link findExecutable}. */
  readonly executable?: string;
  /** Sent in `hello` (logs only). */
  readonly clientName?: string;
  readonly clientVersion?: string;
}

/**
 * A certificate the person chose, as the protocol describes it but with the
 * Base64 fields decoded: `der` and `chain` are what CMS/PAdES code consumes,
 * so they arrive as bytes. `notBefore`/`notAfter` are Unix seconds.
 */
export type Certificate = Omit<WireCertificate, "der" | "chain"> & {
  /** The certificate, DER. */
  readonly der: Uint8Array;
  /** Issuer certificates (DER), leaf excluded, nearest first; best effort, may be empty. */
  readonly chain: readonly Uint8Array[];
};

/** What to sign. */
export interface SignOptions {
  readonly hash: HashName;
  readonly algorithms?: readonly SignatureAlgorithmName[];
  /** Preselect by fingerprint. */
  readonly certificate?: string;
  /**
   * Digest for `certificate` (exactly 32/48/64 bytes for SHA-256/384/512);
   * may run more than once, because the person can switch certificate in the
   * window (the digest of a PAdES/CAdES document covers the certificate, so
   * each switch needs a new one). Throwing aborts the signature and rejects
   * with `Aborted`, the error as `cause`.
   */
  readonly prepare: (
    certificate: Certificate,
    algorithm: SignatureAlgorithmName,
  ) => Uint8Array | ArrayBuffer | Promise<Uint8Array | ArrayBuffer>;
  /** Aborting sends `cancel` and rejects with `Aborted`. */
  readonly signal?: AbortSignal;
}

/** The signature and what it was made with (bytes decoded). */
export interface SignResult {
  readonly certificate: Certificate;
  readonly hash: HashName;
  readonly algorithm: SignatureAlgorithmName;
  /** RSA: the signature block. ECDSA: raw `r || s` (IEEE P1363). */
  readonly signature: Uint8Array;
}
