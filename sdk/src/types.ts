/** Public types of the SDK. Wire types are generated; these are the friendly shapes. */

import type { CurveName, EidasType, KeyStorage } from "./generated/index.js";

export type { CurveName, EidasType, KeyStorage };

/**
 * A hash the digest was computed with.
 *
 * @example
 * const hash: HashAlgorithm = "SHA-256";
 */
export type HashAlgorithm = "SHA-256" | "SHA-384" | "SHA-512";

/**
 * Digest bytes per hash: what `prepare` must return.
 *
 * @example
 * const length: DigestLength<"SHA-384"> = 48;
 */
export type DigestLength<H extends HashAlgorithm = HashAlgorithm> = {
  "SHA-256": 32;
  "SHA-384": 48;
  "SHA-512": 64;
}[H];

/**
 * A signature scheme. RSASSA-PSS uses MGF1 with the same hash and a salt as long
 * as the digest. EdDSA is not offered: it signs messages, not hashes.
 *
 * @example
 * sign({ hash: "SHA-256", algorithm: ["ECDSA", "RSASSA-PSS"], prepare });
 */
export type SignatureAlgorithm = "ECDSA" | "RSASSA-PKCS1-v1_5" | "RSASSA-PSS";

/**
 * Bytes the SDK accepts: a `Uint8Array` (only its view is read) or an
 * `ArrayBuffer`, such as `crypto.subtle.digest()` returns. The SDK always
 * gives bytes back as `Uint8Array`.
 *
 * @example
 * const digest: Bytes = await crypto.subtle.digest("SHA-256", data);
 */
export type Bytes = Uint8Array | ArrayBuffer;

/**
 * The public key.
 *
 * @example
 * const label = key.type === "RSA" ? `RSA ${key.bits}` : `EC ${key.curve}`;
 */
export type KeyDescription =
  | { readonly type: "RSA"; readonly bits: number }
  | { readonly type: "EC"; readonly curve: CurveName };

/**
 * What the certificate declares; the app never claims legal qualification.
 * Your site decides which profiles it accepts.
 *
 * @example
 * const qualified =
 *   /^A[34]$/.test(profile.icpBrasil ?? "") || (profile.eidas?.qualified && profile.eidas.qscd);
 */
export interface CertificateProfile {
  /** ICP-Brasil class ("A3", "S1", "T3"…), "ICP-Brasil" when unknown, absent when not ICP-Brasil. */
  readonly icpBrasil?: string;
  /** Present when the certificate has ETSI qcStatements. */
  readonly eidas?: {
    readonly qualified: boolean;
    readonly qscd: boolean;
    readonly types: readonly EidasType[];
  };
  /** Where the private key lives, as far as the key store can tell. */
  readonly keyStorage: KeyStorage;
}

/**
 * A certificate the person chose (never the list of the computer).
 *
 * @example
 * const [certificate] = await certificates();
 * greet.textContent = `Signing as ${certificate.displayName}, valid until ${certificate.notAfter.toLocaleDateString()}`;
 */
export interface Certificate {
  /** DER bytes: what CMS `certificates` and signing-certificate-v2 need. */
  readonly der: Uint8Array;
  /** Issuers, nearest first, leaf excluded; best effort, may be empty. */
  readonly chain: readonly Uint8Array[];
  /** SHA-256 of `der`, 64 lowercase hex digits. Pass it to `sign({ certificate })`. */
  readonly fingerprint: string;
  /** Holder name as the app shows it. */
  readonly displayName: string;
  /** Issuer common name, else organization. */
  readonly issuerName: string;
  readonly notBefore: Date;
  readonly notAfter: Date;
  readonly key: KeyDescription;
  /** Algorithms this key can produce. */
  readonly algorithms: readonly SignatureAlgorithm[];
  readonly profile: CertificateProfile;
}

/**
 * Passed to `prepare`, with the certificate. Narrowed to what you asked: with
 * `hash: "SHA-384"` and `algorithm: "ECDSA"`, `context.hash` is `"SHA-384"` and
 * `context.algorithm` is `"ECDSA"`.
 *
 * @example
 * prepare: (certificate, { hash, algorithm }) => digestOfSignedAttributes(certificate, hash, algorithm)
 */
export interface PrepareContext<
  H extends HashAlgorithm = HashAlgorithm,
  A extends SignatureAlgorithm = SignatureAlgorithm,
> {
  readonly hash: H;
  /** The algorithm the signature will use (for CMS signatureAlgorithm / algorithm protection). */
  readonly algorithm: A;
}

/**
 * Returns the digest to sign with `certificate`: exactly {@link DigestLength}
 * bytes for the hash. May run more than once if the person switches
 * certificate; only the last digest is signed. Throwing aborts the request.
 *
 * @example
 * const prepare: Prepare<"SHA-256"> = async (certificate, { hash }) =>
 *   crypto.subtle.digest(hash, signedAttributesFor(certificate));
 */
export type Prepare<
  H extends HashAlgorithm = HashAlgorithm,
  A extends SignatureAlgorithm = SignatureAlgorithm,
> = (certificate: Certificate, context: PrepareContext<H, A>) => Bytes | Promise<Bytes>;

/**
 * Options of {@link sign}.
 *
 * @example
 * const options: SignOptions<"SHA-256", "ECDSA"> = {
 *   hash: "SHA-256",
 *   algorithm: "ECDSA",
 *   prepare: (certificate) => digestFor(certificate),
 *   signal: AbortSignal.timeout(120_000),
 * };
 */
export interface SignOptions<
  H extends HashAlgorithm = HashAlgorithm,
  A extends SignatureAlgorithm = SignatureAlgorithm,
> {
  /** The hash `prepare` computes. */
  readonly hash: H;
  /** Acceptable algorithm(s), preferred first. Default: ECDSA for EC keys, PKCS#1 v1.5 for RSA. */
  readonly algorithm?: A | readonly A[];
  /** Preselect a certificate (from an earlier `certificates()`), by object or fingerprint. */
  readonly certificate?: Certificate | string;
  /** Builds the digest once the certificate is known. See {@link Prepare}. */
  readonly prepare: Prepare<H, A>;
  /** Aborts the request (the window closes; the promise rejects with `Aborted`). */
  readonly signal?: AbortSignal;
}

/**
 * The result of {@link sign}: everything a CMS SignerInfo needs.
 *
 * @example
 * const { signature, certificate, algorithm, digest } = await sign({ hash: "SHA-256", prepare });
 */
export interface SignResult<
  H extends HashAlgorithm = HashAlgorithm,
  A extends SignatureAlgorithm = SignatureAlgorithm,
> {
  /** The certificate that signed; the one `prepare` last received. */
  readonly certificate: Certificate;
  readonly hash: H;
  readonly algorithm: A;
  /** RSA: the signature block. ECDSA: raw r‖s (IEEE P1363), each half padded to the curve size. */
  readonly signature: Uint8Array;
  /** The digest that was signed: what `prepare` returned for `certificate`. */
  readonly digest: Uint8Array;
}

/**
 * Options of {@link certificates}.
 *
 * @example
 * await certificates({ algorithm: "ECDSA", signal: controller.signal });
 */
export interface CertificateOptions {
  /** Only certificates whose key can produce one of these. */
  readonly algorithm?: SignatureAlgorithm | readonly SignatureAlgorithm[];
  /** Aborts the request (the window closes; the promise rejects with `Aborted`). */
  readonly signal?: AbortSignal;
}

/**
 * What {@link status} reports. Never opens a window.
 *
 * @example
 * const { ready, problem } = await status();
 * if (!ready) show(errorText(problem, navigator.language));
 */
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
  /** Extension and app present and compatible: `sign()` can work. */
  readonly ready: boolean;
  /**
   * Why not ready, as the error `sign()` would reject with (absent when
   * ready). Pass it to `errorText()` for a localized explanation.
   */
  readonly problem?: StatusProblem;
}

/**
 * The reasons {@link Status} can be not ready.
 *
 * @example
 * if (status.problem === "ExtensionMissing") link.href = installUrl();
 */
export type StatusProblem =
  | "ExtensionMissing"
  | "ExtensionOutdated"
  | "ClientOutdated"
  | "AppMissing"
  | "AppOutdated";

/**
 * The verification code the app shows next to its Sign button.
 *
 * @example
 * const { text, colorIndex, cells } = fingerprint(result.digest);
 */
export interface VerificationCode {
  /** First 8 digest bytes, uppercase hex, 4 groups of 4: "7F3A 9C21 E0B4 55D8". */
  readonly text: string;
  /** Identicon color 0..7 (palette in docs/ux.md §11.1). */
  readonly colorIndex: number;
  /** 25 cells, row-major, true = lit; columns 3–4 mirror 1–0. */
  readonly cells: readonly boolean[];
}
