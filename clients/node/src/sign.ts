/** The `sign.begin` flow: the app asks for a digest once the person picked a certificate. */

import { decodeCertificate } from "./certificate.js";
import { decodeBase64, encodeBase64 } from "./convert.js";
import { checkNeed, toDigestBytes } from "./digest.js";
import { aborted, WebSignError } from "./errors.js";
import type { NeedDigest } from "./generated/index.js";
import type { Exchange, Session } from "./session.js";
import type { HashAlgorithm, SignatureAlgorithm, SignOptions, SignResult } from "./types.js";
import { checkAlgorithms, checkFingerprint, checkHash } from "./validate.js";

/** Runs one signature to its final message. */
export async function runSign(session: Session, options: SignOptions): Promise<SignResult> {
  const hash = checkHash(options.hash);
  const algorithms = checkAlgorithms(options.algorithm);
  const certificate = checkFingerprint(options.certificate);
  if (typeof options.prepare !== "function") {
    throw new WebSignError("InvalidRequest", "prepare must be a function");
  }
  const { signal } = options;
  if (signal?.aborted) throw aborted(signal.reason);

  let latest = 0;
  const exchange: Exchange = session.request(
    "sign.begin",
    begin(hash, algorithms, certificate),
    (event) => {
      if (event.type !== "sign.need_digest" || !Number.isInteger(event.seq)) {
        exchange.fail(new WebSignError("Internal", "the app sent a malformed digest request"));
        return;
      }
      latest = event.seq;
      answer(event).catch((error: unknown) => {
        // A digest for a certificate the person already switched away from is moot.
        if (event.seq !== latest) return;
        exchange.fail(error instanceof WebSignError ? error : aborted(error));
      });
    },
  );

  async function answer(need: NeedDigest): Promise<void> {
    checkNeed(need, hash, algorithms);
    const chosen = decodeCertificate(need.certificate);
    let prepared: unknown;
    try {
      prepared = await options.prepare(chosen, { hash, algorithm: need.algorithm });
    } catch (error) {
      throw aborted(error);
    }
    if (need.seq !== latest) return;
    const digest = toDigestBytes(prepared, hash);
    exchange.send("sign.digest", { seq: need.seq, digest: encodeBase64(digest) });
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

/** `sign.begin` with the optional fields omitted, not `undefined`. */
function begin(
  hash: HashAlgorithm,
  algorithms: SignatureAlgorithm[] | undefined,
  certificate: string | undefined,
): Record<string, unknown> {
  return {
    hash,
    ...(algorithms && { algorithms }),
    ...(certificate && { certificate }),
  };
}
