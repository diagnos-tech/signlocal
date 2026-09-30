import { describe, expect, it } from "vitest";
import { WebSignError } from "../src/errors";
import { encodeFrame, FrameDecoder, MAX_FRAME } from "../src/framing";

const bytes = (message: unknown) => Buffer.from(encodeFrame(message));

describe("encodeFrame", () => {
  it("writes a little-endian u32 length and UTF-8 JSON", () => {
    const frame = bytes({ a: "é" });
    expect(frame.readUInt32LE(0)).toBe(frame.length - 4);
    expect(JSON.parse(frame.subarray(4).toString("utf8"))).toEqual({ a: "é" });
  });

  it("refuses a message over 1 MiB", () => {
    expect(() => encodeFrame({ pad: "x".repeat(MAX_FRAME) })).toThrow(WebSignError);
  });
});

describe("FrameDecoder", () => {
  it("decodes one frame", () => {
    expect(new FrameDecoder().push(bytes({ n: 1 }))).toEqual([{ n: 1 }]);
  });

  it("decodes several frames in one chunk", () => {
    const chunk = Buffer.concat([bytes({ n: 1 }), bytes({ n: 2 }), bytes({ n: 3 })]);
    expect(new FrameDecoder().push(chunk)).toEqual([{ n: 1 }, { n: 2 }, { n: 3 }]);
  });

  it("reassembles a frame split at every byte boundary", () => {
    const whole = Buffer.concat([bytes({ text: "ação" }), bytes({ n: 2 })]);
    for (let cut = 1; cut < whole.length; cut++) {
      const decoder = new FrameDecoder();
      const messages = [
        ...decoder.push(whole.subarray(0, cut)),
        ...decoder.push(whole.subarray(cut)),
      ];
      expect(messages).toEqual([{ text: "ação" }, { n: 2 }]);
    }
  });

  it("splits inside the length header", () => {
    const frame = bytes({ n: 1 });
    const decoder = new FrameDecoder();
    expect(decoder.push(frame.subarray(0, 2))).toEqual([]);
    expect(decoder.push(frame.subarray(2))).toEqual([{ n: 1 }]);
  });

  it("accepts a frame of exactly 1 MiB", () => {
    const filler = "x".repeat(MAX_FRAME - JSON.stringify({ p: "" }).length);
    const messages = new FrameDecoder().push(bytes({ p: filler }));
    expect(messages).toHaveLength(1);
  });

  it("rejects a frame announcing more than 1 MiB before buffering it", () => {
    const header = Buffer.alloc(4);
    header.writeUInt32LE(MAX_FRAME + 1);
    expect(() => new FrameDecoder().push(header)).toThrow(/exceeds 1 MiB/);
  });

  it.each([
    ["invalid JSON", Buffer.from("{nope")],
    ["invalid UTF-8", Buffer.from([0x22, 0xff, 0x22])],
    ["an empty frame", Buffer.alloc(0)],
  ])("rejects %s with an Internal error", (_name, body) => {
    const header = Buffer.alloc(4);
    header.writeUInt32LE(body.length);
    let failure: unknown;
    try {
      new FrameDecoder().push(Buffer.concat([header, body]));
    } catch (error) {
      failure = error;
    }
    expect(failure).toBeInstanceOf(WebSignError);
    expect((failure as WebSignError).code).toBe("Internal");
  });

  it("stays failed after an error", () => {
    const decoder = new FrameDecoder();
    const header = Buffer.alloc(4);
    header.writeUInt32LE(MAX_FRAME + 1);
    expect(() => decoder.push(header)).toThrow();
    expect(() => decoder.push(bytes({ n: 1 }))).toThrow();
  });
});
