/** `fake.verify()`: checks a signature with WebCrypto, as a site's own code would. */

import { viewOf } from "../convert.js";
import type { Bytes, SignResult } from "../types.js";
import type { FakeCredential } from "./certificate.js";

/** A signature of this fake over `data`: false for anything else, never throws. */
export async function verifySignature(
  credentials: readonly FakeCredential[],
  result: SignResult,
  data: Bytes,
): Promise<boolean> {
  const credential = credentials.find((c) => c.wire.fingerprint === result.certificate.fingerprint);
  const bytes = viewOf(data);
  if (!credential || !bytes) return false;
  const { hash } = result;
  // WebCrypto names PSS "RSA-PSS"; the protocol uses the RFC 8017 name.
  const algorithm = result.algorithm === "RSASSA-PSS" ? "RSA-PSS" : result.algorithm;
  const params =
    algorithm === "ECDSA"
      ? { importAs: { name: "ECDSA", namedCurve: "P-256" }, verifyAs: { name: "ECDSA", hash } }
      : {
          importAs: { name: algorithm, hash },
          verifyAs: {
            name: algorithm,
            saltLength: { "SHA-256": 32, "SHA-384": 48, "SHA-512": 64 }[hash],
          },
        };
  try {
    const key = await crypto.subtle.importKey("jwk", credential.jwk, params.importAs, false, [
      "verify",
    ]);
    return await crypto.subtle.verify(
      params.verifyAs,
      key,
      result.signature as BufferSource,
      bytes as BufferSource,
    );
  } catch {
    return false;
  }
}
