/**
 * Validates page requests again in the background (the content script shares
 * a process with the page and cannot be trusted): shape, types, sizes, hash
 * names, digest Base64 length.
 */

import type { PageRequest } from "../generated";

/** The request rebuilt field by field, or null when malformed. */
export function validatePageRequest(value: unknown): PageRequest | null {
  void value;
  throw new Error("unimplemented: SPEC.md §2.4");
}
