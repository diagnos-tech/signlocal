import { describe, expect, it } from "vitest";
import { WebSignError } from "../src/errors";
import { fingerprint } from "../src/fingerprint";

const fromHex = (hex: string) =>
  Uint8Array.from(hex.match(/../g) ?? [], (h) => Number.parseInt(h, 16));
const rowsOf = (cells: readonly boolean[]) =>
  [0, 1, 2, 3, 4]
    .map((r) =>
      cells
        .slice(r * 5, r * 5 + 5)
        .map((c) => (c ? "1" : "0"))
        .join(""),
    )
    .join(" ");
const sha = async (name: string, data: string) =>
  new Uint8Array(await crypto.subtle.digest(name, new TextEncoder().encode(data)));

// Vectors of crates/websign-protocol/SPEC.md section 8.
describe("fingerprint() verification code", () => {
  const vectors: [string, string, string, number, string][] = [
    [
      `7F3A9C21E0B455D8${"00".repeat(24)}`,
      "7F3A 9C21 E0B4 55D8",
      "",
      3,
      "00100 11011 01010 10101 11011",
    ],
    ["00".repeat(32), "0000 0000 0000 0000", "", 0, "00000 00000 00000 00000 00000"],
    ["FF".repeat(32), "FFFF FFFF FFFF FFFF", "", 7, "11111 11111 11111 11111 11111"],
  ];
  for (const [hex, text, , color, rows] of vectors) {
    it(`${text}`, () => {
      const code = fingerprint(fromHex(hex));
      expect(code.text).toBe(text);
      expect(code.colorIndex).toBe(color);
      expect(code.cells).toHaveLength(25);
      expect(rowsOf(code.cells)).toBe(rows);
    });
  }

  it("SHA-256 of the empty string", async () => {
    const code = fingerprint(await sha("SHA-256", ""));
    expect([code.text, code.colorIndex, rowsOf(code.cells)]).toEqual([
      "E3B0 C442 98FC 1C14",
      7,
      "00100 00000 11011 00000 11011",
    ]);
  });

  it("SHA-256 of abc", async () => {
    const code = fingerprint(await sha("SHA-256", "abc"));
    expect([code.text, code.colorIndex, rowsOf(code.cells)]).toEqual([
      "BA78 16BF 8F01 CFEA",
      5,
      "01110 01010 00000 00100 11111",
    ]);
  });

  it("SHA-384 of abc", async () => {
    const code = fingerprint(await sha("SHA-384", "abc"));
    expect([code.text, code.colorIndex, rowsOf(code.cells)]).toEqual([
      "CB00 753F 45A3 5E8B",
      6,
      "10101 01110 10001 00000 00000",
    ]);
  });

  it("only the first 8 bytes matter", () => {
    const a = fingerprint(fromHex(`0102030405060708${"AA".repeat(56)}`));
    const b = fingerprint(fromHex(`0102030405060708${"55".repeat(24)}`));
    expect(a).toEqual(b);
  });

  it("accepts exactly 8 bytes", () => {
    expect(fingerprint(fromHex("7F3A9C21E0B455D8")).text).toBe("7F3A 9C21 E0B4 55D8");
  });

  it("columns 3 and 4 mirror columns 1 and 0", () => {
    const { cells } = fingerprint(fromHex("1234567890ABCDEF"));
    for (let r = 0; r < 5; r++) {
      expect(cells[r * 5 + 3]).toBe(cells[r * 5 + 1]);
      expect(cells[r * 5 + 4]).toBe(cells[r * 5]);
    }
  });

  it("colorIndex is always 0..7", () => {
    for (let b = 0; b < 256; b += 5) {
      const { colorIndex } = fingerprint(Uint8Array.of(b, 0, 0, 0, 0, 0, 0, 0));
      expect(colorIndex).toBe(b >> 5);
    }
  });

  it("handles a view into a larger buffer", () => {
    const backing = new Uint8Array(20).fill(0xee);
    backing.set(fromHex("7F3A9C21E0B455D8"), 4);
    expect(fingerprint(backing.subarray(4, 12)).text).toBe("7F3A 9C21 E0B4 55D8");
  });

  for (const length of [0, 1, 7]) {
    it(`rejects ${length} bytes with InvalidRequest`, () => {
      // The Rust API returns None; the TS API throws a typed error instead (SPEC §9).
      try {
        fingerprint(new Uint8Array(length));
        expect.unreachable("should throw");
      } catch (error) {
        expect(error).toBeInstanceOf(WebSignError);
        expect((error as WebSignError).code).toBe("InvalidRequest");
      }
    });
  }
});

describe("fingerprint() input", () => {
  it("accepts an ArrayBuffer and a subarray view like a Uint8Array", async () => {
    const digest = await sha("SHA-256", "abc");
    const padded = new Uint8Array(40);
    padded.set(digest, 5);
    expect(fingerprint(digest.buffer as ArrayBuffer)).toEqual(fingerprint(digest));
    expect(fingerprint(padded.subarray(5, 37))).toEqual(fingerprint(digest));
  });

  it("rejects anything but bytes with InvalidRequest", () => {
    expect(() => fingerprint("7F3A9C21E0B455D8" as unknown as Uint8Array)).toThrow(
      expect.objectContaining({ code: "InvalidRequest" }),
    );
  });
});
