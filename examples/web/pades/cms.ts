/**
 * The CMS half of PAdES, with PKI.js: the signed attributes whose hash
 * WebeSign signs, and the SignedData that carries the returned signature.
 * Profile: PAdES baseline B-B (ETSI EN 319 142-1), detached, SHA-256.
 */

import type { SignResult } from "@websign/sdk";
import * as asn1js from "asn1js";
import * as pkijs from "pkijs";

const OID = {
  data: "1.2.840.113549.1.7.1",
  signedData: "1.2.840.113549.1.7.2",
  contentType: "1.2.840.113549.1.9.3",
  messageDigest: "1.2.840.113549.1.9.4",
  signingCertificateV2: "1.2.840.113549.1.9.16.2.47",
  sha256: "2.16.840.1.101.3.4.2.1",
  ecdsaWithSha256: "1.2.840.10045.4.3.2",
  rsaEncryption: "1.2.840.113549.1.1.1",
} as const;

const sha256 = async (data: Uint8Array) =>
  new Uint8Array(await crypto.subtle.digest("SHA-256", data as BufferSource));

/** The signed attributes for one certificate, and their DER as a SET: what gets hashed. */
export interface SignedAttributes {
  readonly attributes: pkijs.Attribute[];
  readonly der: Uint8Array;
}

/**
 * contentType, messageDigest and signing-certificate-v2. The last one binds
 * the signature to this certificate, which is why the SDK calls `prepare`
 * only once the person has chosen it.
 */
export async function signedAttributes(
  contentDigest: Uint8Array,
  certificateDer: Uint8Array,
): Promise<SignedAttributes> {
  // ESSCertIDv2 with the default hash (SHA-256) and no issuerSerial, as EN 319 122-1 advises.
  const essCertIdV2 = new asn1js.Sequence({
    value: [new asn1js.OctetString({ valueHex: await sha256(certificateDer) })],
  });
  const unsorted = [
    new pkijs.Attribute({
      type: OID.contentType,
      values: [new asn1js.ObjectIdentifier({ value: OID.data })],
    }),
    new pkijs.Attribute({
      type: OID.messageDigest,
      values: [new asn1js.OctetString({ valueHex: contentDigest })],
    }),
    new pkijs.Attribute({
      type: OID.signingCertificateV2,
      values: [new asn1js.Sequence({ value: [new asn1js.Sequence({ value: [essCertIdV2] })] })],
    }),
  ];
  // DER orders a SET OF by the encodings of its elements.
  const encoded = (a: pkijs.Attribute) => new Uint8Array(a.toSchema().toBER());
  const attributes = unsorted.sort((a, b) => compare(encoded(a), encoded(b)));
  const tagged = new Uint8Array(
    new pkijs.SignedAndUnsignedAttributes({ type: 0, attributes }).toSchema().toBER(),
  );
  tagged[0] = 0x31; // hashed as a universal SET, stored as [0] IMPLICIT (RFC 5652 §5.4)
  return { attributes, der: tagged };
}

function compare(a: Uint8Array, b: Uint8Array): number {
  for (let i = 0; i < Math.max(a.length, b.length); i++) {
    const d = (a[i] ?? 0) - (b[i] ?? 0);
    if (d !== 0) return d;
  }
  return 0;
}

/**
 * The DER ContentInfo to embed in the PDF. WebeSign returns ECDSA as raw r‖s
 * (like WebCrypto); CMS wants the DER Ecdsa-Sig-Value, which PKI.js builds.
 */
export function signedData(result: SignResult, signed: SignedAttributes): Uint8Array {
  const certificate = pkijs.Certificate.fromBER(result.certificate.der);
  const chain = result.certificate.chain.map((der) => pkijs.Certificate.fromBER(der));
  const isEcdsa = result.algorithm === "ECDSA";
  if (!isEcdsa && result.algorithm !== "RSASSA-PKCS1-v1_5") {
    throw new Error(`This example handles ECDSA and PKCS#1 v1.5, not ${result.algorithm}.`);
  }
  const signature = isEcdsa
    ? pkijs.createCMSECDSASignature(result.signature.slice().buffer)
    : result.signature;
  const sha256Algorithm = new pkijs.AlgorithmIdentifier({ algorithmId: OID.sha256 });
  const signerInfo = new pkijs.SignerInfo({
    version: 1,
    sid: new pkijs.IssuerAndSerialNumber({
      issuer: certificate.issuer,
      serialNumber: certificate.serialNumber,
    }),
    digestAlgorithm: sha256Algorithm,
    signedAttrs: new pkijs.SignedAndUnsignedAttributes({ type: 0, attributes: signed.attributes }),
    signatureAlgorithm: isEcdsa
      ? new pkijs.AlgorithmIdentifier({ algorithmId: OID.ecdsaWithSha256 })
      : new pkijs.AlgorithmIdentifier({
          algorithmId: OID.rsaEncryption,
          algorithmParams: new asn1js.Null(),
        }),
    signature: new asn1js.OctetString({ valueHex: signature }),
  });
  const content = new pkijs.SignedData({
    version: 1,
    digestAlgorithms: [sha256Algorithm],
    encapContentInfo: new pkijs.EncapsulatedContentInfo({ eContentType: OID.data }), // detached
    certificates: [certificate, ...chain],
    signerInfos: [signerInfo],
  });
  const info = new pkijs.ContentInfo({
    contentType: OID.signedData,
    content: content.toSchema(true),
  });
  return new Uint8Array(info.toSchema().toBER());
}
