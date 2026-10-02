/** 4-byte little-endian length + UTF-8 JSON, like native messaging. */

import { WebSignError } from "./errors.js";

/** Largest frame in either direction (1 MiB): bounds memory on both sides. */
export const MAX_FRAME = 1024 * 1024;

const HEADER = 4;
const utf8 = new TextDecoder("utf-8", { fatal: true });

/** Frames one message. Refuses a message the app would drop anyway. */
export function encodeFrame(message: unknown): Uint8Array {
  const body = Buffer.from(JSON.stringify(message), "utf8");
  if (body.length > MAX_FRAME) {
    throw new WebSignError("InvalidRequest", `message is ${body.length} bytes; the limit is 1 MiB`);
  }
  const frame = Buffer.allocUnsafe(HEADER + body.length);
  frame.writeUInt32LE(body.length, 0);
  body.copy(frame, HEADER);
  return frame;
}

/**
 * Incremental decoder over stdout chunks. Pipes split and merge writes at
 * will, so it buffers partial frames and can yield several messages per push.
 * After it throws, the stream is unrecoverable and every later push throws.
 */
export class FrameDecoder {
  private buffered: Buffer = Buffer.alloc(0);
  private failure: WebSignError | undefined;

  /** Feeds bytes; returns the complete messages parsed so far. */
  push(chunk: Uint8Array): unknown[] {
    if (this.failure) throw this.failure;
    this.buffered =
      this.buffered.length === 0 ? Buffer.from(chunk) : Buffer.concat([this.buffered, chunk]);
    const messages: unknown[] = [];
    let offset = 0;
    try {
      while (this.buffered.length - offset >= HEADER) {
        const length = this.buffered.readUInt32LE(offset);
        if (length > MAX_FRAME) throw new Error(`frame of ${length} bytes exceeds 1 MiB`);
        const end = offset + HEADER + length;
        if (this.buffered.length < end) break;
        messages.push(JSON.parse(utf8.decode(this.buffered.subarray(offset + HEADER, end))));
        offset = end;
      }
    } catch (error) {
      const reason = error instanceof Error ? error.message : "unreadable frame";
      this.failure = new WebSignError("Internal", `malformed data from the app: ${reason}`);
      this.buffered = Buffer.alloc(0);
      throw this.failure;
    }
    this.buffered = this.buffered.subarray(offset);
    return messages;
  }
}
