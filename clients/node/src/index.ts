/**
 * @websign/desktop — call the WebeSign app from a desktop program.
 *
 * Starts `websign connect` as a child process and speaks the framed protocol
 * on its stdin/stdout. Same flow as the web SDK: the certificate is chosen in
 * the app's window, then `prepare` returns the digest for it. The window
 * shows this program's name (and code signer) as the requester. See
 * docs/architecture/desktop-api.md.
 *
 * @packageDocumentation
 */

export { WebSign } from "./client.js";
export { WebSignError } from "./errors.js";
export type {
  AppInfo,
  CertificateFilter,
  CertificateProfile,
  DiagnosticsTab,
  ErrorCode,
  ErrorDetails,
  HashName,
  KeyDescription,
  SignatureAlgorithmName,
  StatusReply,
} from "./generated/index.js";
export { findExecutable } from "./locate.js";
export type { Certificate, ConnectOptions, SignOptions, SignResult } from "./types.js";
