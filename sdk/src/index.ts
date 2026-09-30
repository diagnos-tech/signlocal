/**
 * @websign/sdk — sign with the visitor's certificate (smart card, USB token or
 * a certificate installed on the computer) from a web page.
 *
 * The page never talks to the app: it posts messages to the WebeSign
 * extension, which adds the page's origin as the browser reports it and
 * forwards them to the app over native messaging. The person confirms every
 * signature in the app's own window.
 *
 * @example
 * import { sign } from "@websign/sdk";
 *
 * const { signature, certificate } = await sign({
 *   hash: "SHA-256",
 *   prepare: (certificate, { hash }) => crypto.subtle.digest(hash, signedAttributesFor(certificate)),
 * });
 *
 * @packageDocumentation
 * @module @websign/sdk
 */

export { certificates } from "./certificates.js";
export { onChange } from "./change.js";
export type { ErrorCode, ErrorDetails } from "./errors.js";
export { isWebSignError, WebSignError } from "./errors.js";
export { fingerprint } from "./fingerprint.js";
export { installUrl } from "./install.js";
export { sign } from "./sign.js";
export { status } from "./status.js";
export type {
  Bytes,
  Certificate,
  CertificateOptions,
  CertificateProfile,
  CurveName,
  DigestLength,
  EidasType,
  HashAlgorithm,
  KeyDescription,
  KeyStorage,
  Prepare,
  PrepareContext,
  SignatureAlgorithm,
  SignOptions,
  SignResult,
  Status,
  StatusProblem,
  VerificationCode,
} from "./types.js";
