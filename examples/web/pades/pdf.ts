/**
 * The PDF half of PAdES, with pdf-lib: reserve room for the signature, find
 * the bytes it covers, and write the CMS into the reserved room. pdf-lib has
 * no signing API, so the signature dictionary is built by hand (ISO 32000-1
 * §12.8, and ETSI EN 319 142-1 for SubFilter ETSI.CAdES.detached).
 */

import { PDFArray, PDFDocument, PDFHexString, PDFName, PDFNumber, PDFString } from "pdf-lib";

/** Room for the CMS: certificates plus signature, with a wide margin (hex doubles it). */
const SIGNATURE_BYTES = 16_384;
/** Replaced by the real offsets once the file is laid out; same width when padded. */
const PLACEHOLDER = "**********";
const latin1 = new TextDecoder("latin1");

/** A PDF saved with an empty signature, and where that signature lives in the file. */
export interface PreparedPdf {
  readonly bytes: Uint8Array;
  /** `[0, start of <hex>, end of <hex>, rest]`: everything except the signature value. */
  readonly byteRange: readonly [number, number, number, number];
}

/** Adds an invisible signature field with an empty value to page 1. */
export async function prepare(
  input: Uint8Array,
  signedAt: Date = new Date(),
): Promise<PreparedPdf> {
  const doc = await PDFDocument.load(input);
  const page = doc.getPage(0);
  const byteRange = PDFArray.withContext(doc.context);
  byteRange.push(PDFNumber.of(0));
  for (let i = 0; i < 3; i++) byteRange.push(PDFName.of(PLACEHOLDER));
  const signature = doc.context.register(
    doc.context.obj({
      Type: "Sig",
      Filter: "Adobe.PPKLite",
      SubFilter: "ETSI.CAdES.detached", // PAdES: a detached CAdES signature
      ByteRange: byteRange,
      Contents: PDFHexString.of("0".repeat(SIGNATURE_BYTES * 2)),
      M: PDFString.fromDate(signedAt), // PAdES keeps the time here, not in the CMS
    }),
  );
  const widget = doc.context.register(
    doc.context.obj({
      Type: "Annot",
      Subtype: "Widget",
      FT: "Sig",
      Rect: [0, 0, 0, 0],
      V: signature,
      T: PDFString.of("Signature1"),
      F: 4,
      P: page.ref,
    }),
  );
  page.node.addAnnot(widget);
  doc.catalog.set(PDFName.of("AcroForm"), doc.context.obj({ SigFlags: 3, Fields: [widget] }));
  // Classic cross-reference table: the offsets found below stay valid.
  return fillByteRange(await doc.save({ useObjectStreams: false }));
}

function fillByteRange(bytes: Uint8Array): PreparedPdf {
  const text = latin1.decode(bytes);
  const marker = `/ByteRange [ 0 /${PLACEHOLDER} /${PLACEHOLDER} /${PLACEHOLDER} ]`;
  const at = text.lastIndexOf(marker);
  const start = text.indexOf("/Contents <", at) + "/Contents ".length;
  const end = text.indexOf(">", start) + 1;
  if (at < 0 || start < at || end <= start) throw new Error("Signature placeholder not found.");
  const byteRange = [0, start, end, bytes.length - end] as const;
  const filled = `/ByteRange [${byteRange.join(" ")}]`.padEnd(marker.length, " ");
  const out = bytes.slice();
  out.set(new TextEncoder().encode(filled), at);
  return { bytes: out, byteRange };
}

/** The bytes the signature covers: the whole file minus the `<hex>` value. */
export function signedContent({ bytes, byteRange: [, start, end] }: PreparedPdf): Uint8Array {
  const out = new Uint8Array(bytes.length - (end - start));
  out.set(bytes.subarray(0, start));
  out.set(bytes.subarray(end), start);
  return out;
}

/** Writes the DER CMS into the reserved `<hex>` (zero-padded, as PDF readers expect). */
export function embed(
  { bytes, byteRange: [, start, end] }: PreparedPdf,
  cms: Uint8Array,
): Uint8Array {
  const hex = Array.from(cms, (b) => b.toString(16).padStart(2, "0")).join("");
  const room = end - start - 2;
  if (hex.length > room)
    throw new Error(`The signature needs ${hex.length / 2} bytes; ${room / 2} reserved.`);
  const out = bytes.slice();
  out.set(new TextEncoder().encode(hex.padEnd(room, "0")), start + 1);
  return out;
}

/** A one-page sample document, so the example runs without a file. */
export async function samplePdf(): Promise<Uint8Array> {
  const doc = await PDFDocument.create();
  doc.addPage([420, 200]).drawText("Contract: I agree to the terms.", { x: 40, y: 120, size: 16 });
  return doc.save();
}
