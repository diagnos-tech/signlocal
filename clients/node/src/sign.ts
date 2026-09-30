/** The `sign.begin` flow: the app asks for a digest once the person picked a certificate. */

import { decodeCertificate } from "./certificate.js";
import { decodeBase64, encodeBase64 } from "./convert.js";
import { WebSignError, withCause } from "./errors.js";
import type { HashName, SignatureAlgorithmName } from "./generated/index.js";
import type { Exchange, Session } from "./session.js";
import type { Certificate, SignOptions, SignResult } from "./types.js";

const DIGEST_LENGTH: Record<HashName, number> = { "SHA-256": 32, "SHA-384": 48, "SHA-512": 64 };
const ALGORITHMS: readonly string[] = ["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"];

/** Runs one signature to its final message. */
export async function runSign(session: Session, options: SignOptions): Promise<SignResult> {
  const fields = validate(options);
  const { signal } = options;
  if (signal?.aborted) throw aborted(signal.reason);

  let latest = 0;
  const exchange: Exchange = session.request("sign.begin", fields, (event) => {
    if (event.type !== "sign.need_digest" || !Number.isInteger(event.seq)) {
      exchange.fail(new WebSignError("Internal", "the app sent a malformed digest request"));
      return;
    }
    latest = event.seq;
    let certificate: Certificate;
    try {
      certificate = decodeCertificate(event.certificate);
    } catch (error) {
      exchange.fail(error as WebSignError);
      return;
    }
    answer(event.seq, certificate, event.algorithm).catch((error: unknown) => {
      // A digest for a certificate the person already switched away from is moot.
      if (event.seq !== latest) return;
      exchange.fail(error instanceof WebSignError ? error : aborted(error));
    });
  });

  async function answer(
    seq: number,
    certificate: Certificate,
    algorithm: SignatureAlgorithmName,
  ): Promise<void> {
    let prepared: Uint8Array | ArrayBuffer;
    try {
      prepared = await options.prepare(certificate, algorithm);
    } catch (error) {
      throw aborted(error);
    }
    const digest = toBytes(prepared);
    if (seq !== latest) return;
    const expected = DIGEST_LENGTH[options.hash];
    if (digest.length !== expected) {
      throw new WebSignError(
        "InvalidRequest",
        `digest is ${digest.length} bytes; ${options.hash} requires ${expected}`,
      );
    }
    exchange.send("sign.digest", { seq, digest: encodeBase64(digest) });
  }

  const onAbort = () => exchange.fail(aborted(signal?.reason));
  signal?.addEventListener("abort", onAbort, { once: true });
  try {
    const reply = await exchange.result;
    if (reply.type !== "sign.result" || typeof reply.algorithm !== "string") {
      throw new WebSignError("Internal", `unexpected reply ${reply.type} to sign.begin`);
    }
    return {
      certificate: decodeCertificate(reply.certificate),
      hash: reply.hash,
      algorithm: reply.algorithm,
      signature: decodeBase64(reply.signature),
    };
  } finally {
    signal?.removeEventListener("abort", onAbort);
  }
}

function validate(options: SignOptions): Record<string, unknown> {
  if (!Object.hasOwn(DIGEST_LENGTH, options.hash)) {
    throw new WebSignError("InvalidRequest", `unsupported hash: ${String(options.hash)}`);
  }
  if (typeof options.prepare !== "function") {
    throw new WebSignError("InvalidRequest", "prepare must be a function");
  }
  const fields: Record<string, unknown> = { hash: options.hash };
  if (options.algorithms !== undefined) {
    const unique = [...new Set(options.algorithms)];
    if (unique.length === 0 || unique.some((name) => !ALGORITHMS.includes(name))) {
      throw new WebSignError(
        "InvalidRequest",
        "algorithms must be a non-empty list of known names",
      );
    }
    fields.algorithms = unique;
  }
  if (options.certificate !== undefined) fields.certificate = options.certificate;
  return fields;
}

function toBytes(value: Uint8Array | ArrayBuffer): Uint8Array {
  if (value instanceof Uint8Array) return value;
  if (value instanceof ArrayBuffer) return new Uint8Array(value);
  throw new WebSignError("InvalidRequest", "prepare must return a Uint8Array");
}

function aborted(cause: unknown): WebSignError {
  return withCause(new WebSignError("Aborted", "the caller cancelled the request"), cause);
}
