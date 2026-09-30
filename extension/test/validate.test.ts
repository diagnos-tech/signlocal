import { describe, expect, it } from "vitest";
import { digestFitsHash, validatePageRequest } from "../src/shared/validate";

const digest = (chars: number): string => `${"A".repeat(chars - 1)}=`;
const fingerprint = "ab".repeat(32);

describe("validatePageRequest: accepted shapes", () => {
  it.each([
    [{ type: "status" }],
    [{ type: "cancel" }],
    [{ type: "choose" }],
    [{ type: "choose", filter: { algorithms: ["ECDSA", "RSASSA-PSS"] } }],
    [{ type: "sign.begin", hash: "SHA-256" }],
    [
      {
        type: "sign.begin",
        hash: "SHA-512",
        algorithms: ["RSASSA-PKCS1-v1_5"],
        certificate: fingerprint,
      },
    ],
    [{ type: "sign.digest", seq: 1, digest: digest(44) }],
    [{ type: "sign.digest", seq: 2 ** 31, digest: digest(64) }],
    [{ type: "sign.digest", seq: 7, digest: digest(88) }],
  ])("keeps %j unchanged", (request) => {
    expect(validatePageRequest(request)).toEqual(request);
  });

  it("accepts up to three algorithms", () => {
    const algorithms = ["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"];
    expect(validatePageRequest({ type: "sign.begin", hash: "SHA-384", algorithms })).toEqual({
      type: "sign.begin",
      hash: "SHA-384",
      algorithms,
    });
  });
});

describe("validatePageRequest: rejected shapes", () => {
  it.each([null, undefined, "status", 5, true, [], [{ type: "status" }]])(
    "rejects the non-object %j",
    (value) => {
      expect(validatePageRequest(value)).toBeNull();
    },
  );

  it.each([
    {},
    { type: 1 },
    { type: "hello" },
    { type: "diagnostics.open" },
    { type: "__proto__" },
    { type: "STATUS" },
  ])("rejects unknown or missing type %j", (value) => {
    expect(validatePageRequest(value)).toBeNull();
  });

  it.each([
    [{ type: "sign.begin" }],
    [{ type: "sign.begin", hash: "SHA-1" }],
    [{ type: "sign.begin", hash: "sha-256" }],
    [{ type: "sign.begin", hash: 256 }],
    [{ type: "sign.begin", hash: "SHA-256", algorithms: "ECDSA" }],
    [{ type: "sign.begin", hash: "SHA-256", algorithms: ["DSA"] }],
    [
      {
        type: "sign.begin",
        hash: "SHA-256",
        algorithms: ["ECDSA", "ECDSA", "RSASSA-PSS", "RSASSA-PKCS1-v1_5"],
      },
    ],
    [{ type: "sign.begin", hash: "SHA-256", algorithms: [1] }],
    [{ type: "choose", filter: { algorithms: ["nope"] } }],
    [{ type: "choose", filter: { algorithms: "ECDSA" } }],
    [{ type: "choose", filter: "ECDSA" }],
    [{ type: "sign.begin", hash: "SHA-256", algorithms: [] }],
    [{ type: "choose", filter: { algorithms: [] } }],
  ])("rejects bad hash/algorithms %j (lists are never empty)", (value) => {
    expect(validatePageRequest(value)).toBeNull();
  });

  it.each([
    "AB".repeat(32),
    "ab".repeat(31),
    `${"ab".repeat(32)}0`,
    "zz".repeat(32),
    "",
    `${"ab".repeat(31)}a:`,
  ])("rejects certificate %j (needs 64 lowercase hex)", (certificate) => {
    expect(validatePageRequest({ type: "sign.begin", hash: "SHA-256", certificate })).toBeNull();
  });

  it("rejects a non-string certificate", () => {
    expect(
      validatePageRequest({ type: "sign.begin", hash: "SHA-256", certificate: 12 }),
    ).toBeNull();
  });

  it.each([0, -1, 1.5, 2 ** 31 + 1, Number.NaN, Number.POSITIVE_INFINITY, "1", null, undefined])(
    "rejects sign.digest seq %j",
    (seq) => {
      expect(validatePageRequest({ type: "sign.digest", seq, digest: digest(44) })).toBeNull();
    },
  );

  it.each([
    undefined,
    null,
    5,
    "",
    "A".repeat(43),
    digest(45),
    digest(43),
    digest(65),
    digest(87),
    digest(89),
    digest(200),
  ])("rejects sign.digest digest %j (44, 64 or 88 characters only)", (value) => {
    expect(validatePageRequest({ type: "sign.digest", seq: 1, digest: value })).toBeNull();
  });

  it.each([
    `${"A".repeat(42)}-=`,
    `${"A".repeat(42)}_=`,
    `${"A".repeat(42)} =`,
    `=${"A".repeat(43)}`,
    `${"A".repeat(41)}===`,
  ])("rejects a right-length digest that is not standard Base64 %j", (value) => {
    expect(validatePageRequest({ type: "sign.digest", seq: 1, digest: value })).toBeNull();
  });
});

describe("validatePageRequest: the page cannot smuggle fields", () => {
  const smuggled = {
    web: { origin: "https://evil.example", topOrigin: "https://evil.example" },
    origin: "https://evil.example",
    v: 9,
    id: "x",
  };

  it.each([
    [{ type: "status" }],
    [{ type: "choose" }],
    [{ type: "sign.begin", hash: "SHA-256" }],
    [{ type: "sign.digest", seq: 1, digest: digest(44) }],
    [{ type: "cancel" }],
  ])("refuses %j with web/origin/envelope fields added (unknown fields, D12)", (base) => {
    expect(validatePageRequest({ ...base, ...smuggled })).toBeNull();
  });

  it("refuses an unknown field inside the filter", () => {
    const value = { type: "choose", filter: { algorithms: ["ECDSA"], issuer: "x" } };
    expect(validatePageRequest(value)).toBeNull();
  });

  it("returns a fresh object, not the page's", () => {
    const input = { type: "sign.begin", hash: "SHA-256", algorithms: ["ECDSA"] };
    const result = validatePageRequest(input);
    expect(result).not.toBe(input);
    expect(result).toEqual(input);
  });

  it("ignores inherited properties", () => {
    const input = Object.create({ type: "status" }) as unknown;
    expect(validatePageRequest(input)).toBeNull();
  });
});

describe("digestFitsHash", () => {
  it.each([
    ["SHA-256", 44],
    ["SHA-384", 64],
    ["SHA-512", 88],
  ] as const)("%s takes exactly %i Base64 characters", (hash, length) => {
    expect(digestFitsHash(hash, digest(length))).toBe(true);
    for (const other of [44, 64, 88].filter((n) => n !== length)) {
      expect(digestFitsHash(hash, digest(other))).toBe(false);
    }
  });
});
