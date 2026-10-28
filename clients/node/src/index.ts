/**
 * @signlocal/desktop — call the signlocal app from a desktop program.
 *
 * Starts `signlocal connect` as a child process and speaks the framed protocol
 * on its stdin/stdout. Same flow as the web SDK: the certificate is chosen in
 * the app's window, then `prepare` returns the digest for it. The window
 * shows this program's name (and code signer) as the requester. See
 * docs/architecture/desktop-api.md.
 *
 * @packageDocumentation
 */

export { Signlocal } from "./client.js";
export { isSignlocalError, SignlocalError } from "./errors.js";
export type {
  AppInfo,
  CertificateProfile,
  CurveName,
  DiagnosticsTab,
  EidasType,
  ErrorCode,
  ErrorDetails,
  KeyDescription,
  KeyStorage,
  StatusReply,
} from "./generated/index.js";
export { findExecutable } from "./locate.js";
export type {
  Certificate,
  CertificateOptions,
  ConnectOptions,
  HashAlgorithm,
  PrepareContext,
  SignatureAlgorithm,
  SignOptions,
  SignResult,
} from "./types.js";
