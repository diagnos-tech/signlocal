/** One connection to the app. */

import type {
  Certificate,
  CertificateFilter,
  DiagnosticsTab,
  HashName,
  SignatureAlgorithmName,
  StatusReply,
} from "./generated";

/** How to start the app. */
export interface ConnectOptions {
  /** Path of `websign`; default {@link findExecutable}. */
  readonly executable?: string;
  /** Sent in `hello` (logs only). */
  readonly clientName?: string;
  readonly clientVersion?: string;
}

/** What to sign. */
export interface SignOptions {
  readonly hash: HashName;
  readonly algorithms?: readonly SignatureAlgorithmName[];
  /** Preselect by fingerprint. */
  readonly certificate?: string;
  /** Digest for `certificate`; may run more than once. Throwing aborts. */
  readonly prepare: (
    certificate: Certificate,
    algorithm: SignatureAlgorithmName,
  ) => Uint8Array | Promise<Uint8Array>;
  readonly signal?: AbortSignal;
}

/** The signature and what it was made with (bytes decoded). */
export interface SignResult {
  readonly certificate: Certificate;
  readonly hash: HashName;
  readonly algorithm: SignatureAlgorithmName;
  readonly signature: Uint8Array;
}

/** A running `websign connect` child. Call {@link WebSign.close} when done. */
export class WebSign {
  private constructor() {}

  /** Starts the app and negotiates the protocol. Rejects with `AppMissing` when not installed. */
  static connect(options?: ConnectOptions): Promise<WebSign> {
    void options;
    throw new Error("unimplemented: SPEC.md §2");
  }

  /** `status`. */
  status(): Promise<StatusReply> {
    throw new Error("unimplemented: SPEC.md §3");
  }

  /** `choose`. */
  certificates(filter?: CertificateFilter): Promise<Certificate[]> {
    void filter;
    throw new Error("unimplemented: SPEC.md §3");
  }

  /** `sign.begin` … `sign.result`. */
  sign(options: SignOptions): Promise<SignResult> {
    void options;
    throw new Error("unimplemented: SPEC.md §3");
  }

  /** `diagnostics.open`. */
  openDiagnostics(tab?: DiagnosticsTab): Promise<void> {
    void tab;
    throw new Error("unimplemented: SPEC.md §3");
  }

  /** Closes stdin; the app exits. */
  close(): Promise<void> {
    throw new Error("unimplemented: SPEC.md §2");
  }
}
