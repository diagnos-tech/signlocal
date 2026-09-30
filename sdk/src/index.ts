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

export { certificates } from "./certificates";
export { onChange } from "./change";
export type { ErrorCode } from "./errors";
export { WebSignError } from "./errors";
export { fingerprint } from "./fingerprint";
export { installUrl } from "./install";
export { sign } from "./sign";
export { status } from "./status";
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
} from "./types";
