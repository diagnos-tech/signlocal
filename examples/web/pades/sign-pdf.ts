/**
 * PAdES with @websign/sdk, end to end: the PDF library computes what to
 * sign, SignLocal signs its hash with the person's certificate, and the raw
 * signature goes back into the CMS inside the PDF.
 */

import { type Certificate, sign } from "@websign/sdk";
import { type SignedAttributes, signedAttributes, signedData } from "./cms";
import { embed, prepare, signedContent } from "./pdf";

export interface SignedPdf {
  readonly pdf: Uint8Array;
  readonly certificate: Certificate;
  /** What `prepare` returned for the signing certificate: show it as a verification code. */
  readonly digest: Uint8Array;
}

export async function signPdf(input: Uint8Array, signal?: AbortSignal): Promise<SignedPdf> {
  // 1. Reserve room for the signature and hash everything else.
  const prepared = await prepare(input);
  const contentDigest = new Uint8Array(
    await crypto.subtle.digest("SHA-256", signedContent(prepared) as BufferSource),
  );

  // 2. SignLocal: the person chooses a certificate, then `prepare` builds the signed
  //    attributes for it (they name the certificate) and returns their hash. If the
  //    person switches certificate, `prepare` runs again: keep one set per certificate.
  const byCertificate = new Map<string, SignedAttributes>();
  const result = await sign({
    hash: "SHA-256",
    algorithm: ["ECDSA", "RSASSA-PKCS1-v1_5"],
    ...(signal && { signal }),
    prepare: async (certificate, { hash }) => {
      const attributes = await signedAttributes(contentDigest, certificate.der);
      byCertificate.set(certificate.fingerprint, attributes);
      return crypto.subtle.digest(hash, attributes.der as BufferSource);
    },
  });

  // 3. Plug the raw signature into the CMS, and the CMS into the PDF.
  const attributes = byCertificate.get(result.certificate.fingerprint);
  if (!attributes) throw new Error("No signed attributes for the signing certificate.");
  return {
    pdf: embed(prepared, signedData(result, attributes)),
    certificate: result.certificate,
    digest: result.digest,
  };
}
