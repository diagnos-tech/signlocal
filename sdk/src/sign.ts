import { follow } from "./channel.js";
import { fromBase64, toBase64, toCertificate } from "./convert.js";
import { checkNeed, toDigestBytes } from "./digest.js";
import { WebSignError } from "./errors.js";
import type {
  HashName,
  NeedDigest,
  PageRequest,
  SignatureAlgorithmName,
} from "./generated/index.js";
import { ask, connect, readReply } from "./request.js";
import type { Certificate, PrepareContext, SignOptions, SignResult } from "./types.js";
import { checkAlgorithms, checkFingerprint, checkHash } from "./validate.js";

/**
 * Signs a digest the page prepares for the certificate the person chooses.
 * One confirmation window per call: the certificate is chosen there and
 * `prepare` runs afterwards, because PAdES/CAdES put the certificate inside
 * the signed attributes. `prepare` runs again if the person switches
 * certificate; only the digest of the last one is signed.
 *
 * @example
 * const { signature, certificate } = await sign({
 *   hash: "SHA-256",
 *   prepare: (certificate, { algorithm }) => digestOfSignedAttributes(certificate, algorithm),
 * });
 *
 * @throws {WebSignError} `InvalidRequest` (bad options or digest length),
 * `Aborted` (signal fired or `prepare` threw; the original is `cause`),
 * `ExtensionMissing`, `ClientOutdated`, `UserCancelled`, `PinLocked`, `Timeout`…
 */
export async function sign(options: SignOptions): Promise<SignResult> {
  if (typeof options !== "object" || options === null) {
    throw new WebSignError(
      "InvalidRequest",
      "sign() needs an options object with hash and prepare.",
    );
  }
  const hash = checkHash(options.hash);
  const algorithms = checkAlgorithms(options.algorithm);
  const certificate = checkFingerprint(options.certificate);
  if (typeof options.prepare !== "function") {
    throw new WebSignError(
      "InvalidRequest",
      "prepare must be a function returning the digest to sign.",
    );
  }
  const { prepare, signal } = options;
  await connect(signal);

  // `run` ends the request: on the caller's signal, or when a digest cannot be sent.
  const run = new AbortController();
  const stop = () => run.abort();
  signal?.addEventListener("abort", stop, { once: true });
  let failure: unknown;
  let latest = 0;
  let open = true;

  const answer = async (need: NeedDigest, id: string): Promise<void> => {
    if (need.seq <= latest) return;
    latest = need.seq;
    // A newer need_digest, the end of the request or an abort makes this answer moot.
    const current = () => open && need.seq === latest && !run.signal.aborted;
    try {
      checkNeed(need, hash, algorithms);
      const chosen = readReply(() => toCertificate(need.certificate));
      const digest = await callPrepare(prepare, chosen, { hash, algorithm: need.algorithm });
      if (!current()) return;
      const bytes = toDigestBytes(digest, hash);
      follow(id, { type: "sign.digest", seq: need.seq, digest: toBase64(bytes) });
    } catch (error) {
      if (!current()) return;
      failure = error;
      run.abort();
    }
  };

  try {
    const reply = await ask(begin(hash, algorithms, certificate), "sign.result", {
      signal: run.signal,
      onNeedDigest: (need, id) => void answer(need, id),
    });
    return readReply(() => ({
      certificate: toCertificate(reply.certificate),
      hash: reply.hash,
      algorithm: reply.algorithm,
      signature: fromBase64(reply.signature),
    }));
  } catch (error) {
    throw failure ?? error;
  } finally {
    open = false;
    signal?.removeEventListener("abort", stop);
  }
}

/** `sign.begin` with the optional fields omitted, not `undefined`. */
function begin(
  hash: HashName,
  algorithms: SignatureAlgorithmName[] | undefined,
  certificate: string | undefined,
): PageRequest {
  return {
    type: "sign.begin",
    hash,
    ...(algorithms && { algorithms }),
    ...(certificate && { certificate }),
  };
}

/**
 * Runs the site's `prepare`. Its error becomes `cause` and stays out of the
 * message: site errors may quote the document being signed.
 */
async function callPrepare(
  prepare: SignOptions["prepare"],
  certificate: Certificate,
  context: PrepareContext,
): Promise<unknown> {
  try {
    return await prepare(certificate, context);
  } catch (cause) {
    const error = new WebSignError(
      "Aborted",
      "prepare() threw, so the request was cancelled; the original error is the cause.",
    );
    error.cause = cause;
    throw error;
  }
}
