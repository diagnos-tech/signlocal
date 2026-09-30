import { toCertificate } from "./convert.js";
import { WebSignError } from "./errors.js";
import { ask, connect, readReply } from "./request.js";
import type { Certificate, CertificateOptions } from "./types.js";
import { checkAlgorithms } from "./validate.js";

/**
 * The certificate the person chooses in the app's window — never the list of
 * the computer. A remembered site gets the certificates it already used,
 * without a window.
 *
 * Use it to learn who is signing (name, issuer, profile) before you build the
 * document; pass the result to `sign({ certificate })` to skip the choice.
 *
 * @example
 * const [certificate] = await certificates({ algorithm: "ECDSA" });
 *
 * @throws {WebSignError} `ExtensionMissing`, `AppMissing`, `AppOutdated`,
 * `UserCancelled`, `NoCertificates`, `Timeout`, `Busy`, `Aborted`…
 */
export async function certificates(options: CertificateOptions = {}): Promise<Certificate[]> {
  const algorithms = checkAlgorithms(options.algorithm);
  await connect(options.signal);
  const reply = await ask(
    { type: "choose", ...(algorithms && { filter: { algorithms } }) },
    "choose.result",
    { signal: options.signal },
  );
  const list = readReply(() => reply.certificates.map(toCertificate));
  // The app reports an empty choice as NoCertificates; an empty list would be a bug there.
  if (list.length === 0) throw new WebSignError("NoCertificates", "No certificate was chosen.");
  return list;
}
