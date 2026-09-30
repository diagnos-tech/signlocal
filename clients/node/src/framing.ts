/** 4-byte little-endian length + UTF-8 JSON, like native messaging. */

/** Largest frame accepted from the app (1 MiB). */
export const MAX_FRAME = 1024 * 1024;

/** Frames one message. */
export function encodeFrame(message: unknown): Uint8Array {
  void message;
  throw new Error("unimplemented: SPEC.md §2");
}

/** Incremental decoder over stdout chunks. */
export class FrameDecoder {
  /** Feeds bytes; returns the complete messages parsed so far. */
  push(chunk: Uint8Array): unknown[] {
    void chunk;
    throw new Error("unimplemented: SPEC.md §2");
  }
}
