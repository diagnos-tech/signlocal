/** Wire ↔ public conversions: Base64 ↔ bytes, Unix seconds ↔ Date. */

import type { Certificate as WireCertificate } from "./generated";
import type { Certificate } from "./types";

/** Strict padded standard Base64 → bytes (throws InvalidRequest-worthy errors on anything else). */
export function fromBase64(text: string): Uint8Array {
  void text;
  throw new Error("unimplemented: SPEC.md §3");
}

/** Bytes → padded standard Base64. */
export function toBase64(bytes: Uint8Array): string {
  void bytes;
  throw new Error("unimplemented: SPEC.md §3");
}

/** Wire certificate → public certificate. */
export function toCertificate(wire: WireCertificate): Certificate {
  void wire;
  throw new Error("unimplemented: SPEC.md §3");
}
