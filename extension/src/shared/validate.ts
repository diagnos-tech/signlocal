/**
 * Validates page requests: shape, types, sizes, hash names, digest Base64
 * length. Runs twice: in the content script, so nothing the page invented
 * travels further, and again in the background, because the content script
 * shares a process with the page and cannot be trusted. Unknown fields are
 * refused, like the app does (protocol.md §2, D12), so nothing rides along
 * unexamined.
 */

import type {
  CertificateFilter,
  HashName,
  PageRequest,
  SignatureAlgorithmName,
} from "../generated";

const HASHES = ["SHA-256", "SHA-384", "SHA-512"] as const;
const ALGORITHMS: readonly SignatureAlgorithmName[] = ["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"];
const MAX_ALGORITHMS = 3;
const MAX_SEQ = 2 ** 31;
const FINGERPRINT = /^[0-9a-f]{64}$/;
/** Base64 text length of each hash's digest (32, 48, 64 bytes); the app checks the padding. */
const DIGEST_TEXT_LENGTH: Readonly<Record<HashName, number>> = {
  "SHA-256": 44,
  "SHA-384": 64,
  "SHA-512": 88,
};
const DIGEST_LENGTHS = Object.values(DIGEST_TEXT_LENGTH);
const BASE64 = /^[A-Za-z0-9+/]+={0,2}$/;

type Fields = Readonly<Record<string, unknown>>;

/**
 * A copy of the record's own fields when it is a plain object with no field
 * outside `allowed`. Copying keeps inherited properties (and anything else
 * that is not an own enumerable field) from being read later.
 */
function record(value: unknown, allowed: readonly string[]): Fields | null {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return null;
  const keys = Object.keys(value);
  if (!keys.every((key) => allowed.includes(key))) return null;
  return Object.fromEntries(keys.map((key) => [key, (value as Fields)[key]]));
}

function isHash(value: unknown): value is (typeof HASHES)[number] {
  return HASHES.some((hash) => hash === value);
}

/** A non-empty list of known algorithm names, or null. */
function algorithmList(value: unknown): SignatureAlgorithmName[] | null {
  if (!Array.isArray(value) || value.length === 0 || value.length > MAX_ALGORITHMS) return null;
  const names: SignatureAlgorithmName[] = [];
  for (const item of value) {
    const known = ALGORITHMS.find((name) => name === item);
    if (known === undefined) return null;
    names.push(known);
  }
  return names;
}

function choose(fields: Fields): PageRequest | null {
  if (fields.filter === undefined) return { type: "choose" };
  const filter = record(fields.filter, ["algorithms"]);
  if (filter === null) return null;
  const clean: CertificateFilter = {};
  if (filter.algorithms !== undefined) {
    const algorithms = algorithmList(filter.algorithms);
    if (algorithms === null) return null;
    clean.algorithms = algorithms;
  }
  return { type: "choose", filter: clean };
}

function signBegin(fields: Fields): PageRequest | null {
  if (!isHash(fields.hash)) return null;
  const request: PageRequest & { type: "sign.begin" } = { type: "sign.begin", hash: fields.hash };
  if (fields.algorithms !== undefined) {
    const algorithms = algorithmList(fields.algorithms);
    if (algorithms === null) return null;
    request.algorithms = algorithms;
  }
  if (fields.certificate !== undefined) {
    if (typeof fields.certificate !== "string" || !FINGERPRINT.test(fields.certificate)) {
      return null;
    }
    request.certificate = fields.certificate;
  }
  return request;
}

function signDigest(fields: Fields): PageRequest | null {
  const { seq, digest } = fields;
  if (typeof seq !== "number" || !Number.isInteger(seq) || seq < 1 || seq > MAX_SEQ) return null;
  if (typeof digest !== "string") return null;
  if (!DIGEST_LENGTHS.includes(digest.length) || !BASE64.test(digest)) return null;
  return { type: "sign.digest", seq, digest };
}

/**
 * Whether `digest` has exactly the length of a `hash` digest. The shape
 * check above cannot know the hash (it lives in the earlier `sign.begin`),
 * so the background, which remembers it per request, checks this too.
 */
export function digestFitsHash(hash: HashName, digest: string): boolean {
  return digest.length === DIGEST_TEXT_LENGTH[hash];
}

/** The request rebuilt field by field, or null when malformed. */
export function validatePageRequest(value: unknown): PageRequest | null {
  const header = record(value, [
    "type",
    "filter",
    "hash",
    "algorithms",
    "certificate",
    "seq",
    "digest",
  ]);
  if (header === null) return null;
  const allowed = (names: readonly string[]) => record(header, ["type", ...names]);
  switch (header.type) {
    case "status":
      return allowed([]) && { type: "status" };
    case "cancel":
      return allowed([]) && { type: "cancel" };
    case "choose": {
      const fields = allowed(["filter"]);
      return fields && choose(fields);
    }
    case "sign.begin": {
      const fields = allowed(["hash", "algorithms", "certificate"]);
      return fields && signBegin(fields);
    }
    case "sign.digest": {
      const fields = allowed(["seq", "digest"]);
      return fields && signDigest(fields);
    }
    default:
      return null;
  }
}
