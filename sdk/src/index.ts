/**
 * @websign/sdk — sign with the user's certificate from a web page.
 *
 * The page never talks to the app: it posts messages to the WebeSign
 * extension, which adds the page's origin as the browser reports it and
 * forwards them to the app over native messaging. The person confirms every
 * signature in the app's own window. See docs/architecture/web-api.md.
 *
 * @packageDocumentation
 */

export { certificates } from "./certificates.js";
export { onChange } from "./change.js";
export type { ErrorCode } from "./errors.js";
export { WebSignError } from "./errors.js";
export { fingerprint } from "./fingerprint.js";
export { installUrl } from "./install.js";
export { sign } from "./sign.js";
export { status } from "./status.js";
export type {
  Certificate,
  CertificateOptions,
  CertificateProfile,
  HashAlgorithm,
  KeyDescription,
  PrepareContext,
  SignatureAlgorithm,
  SignOptions,
  SignResult,
  Status,
  VerificationCode,
} from "./types.js";
