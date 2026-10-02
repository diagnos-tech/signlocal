import { WebSignError } from "./errors.js";

const CANONICAL_BASE64 = /^(?:[A-Za-z0-9+/]{4})*(?:[A-Za-z0-9+/]{2}==|[A-Za-z0-9+/]{3}=)?$/;

/** Decodes the protocol's canonical padded Base64; anything else is an app bug, not caller input. */
export function decodeBase64(value: unknown): Uint8Array {
  if (typeof value !== "string" || !CANONICAL_BASE64.test(value)) {
    throw new WebSignError("Internal", "the app sent malformed Base64");
  }
  return new Uint8Array(Buffer.from(value, "base64"));
}

/** Encodes bytes as canonical padded Base64. */
export function encodeBase64(bytes: Uint8Array): string {
  return Buffer.from(bytes.buffer, bytes.byteOffset, bytes.byteLength).toString("base64");
}
