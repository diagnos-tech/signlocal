/** Public types of the fake that more than one of its files use. */

import type { HashAlgorithm, SignatureAlgorithm } from "../types.js";

/**
 * Who the person picks: an index into the fake's certificates or a fingerprint.
 *
 * @example
 * fake.choose(1); // the second certificate
 */
export type CertificatePick = number | string;

/**
 * A request the page made, as the fake saw it: assert on these in your tests.
 *
 * @example
 * expect(fake.requests.at(-1)).toMatchObject({ type: "sign", hash: "SHA-256" });
 */
export interface FakeRequest {
  readonly type: "status" | "certificates" | "sign";
  /** `sign`: the hash asked for. */
  readonly hash?: HashAlgorithm;
  /** The algorithms the page accepted, when it restricted them. */
  readonly algorithms?: readonly SignatureAlgorithm[];
  /** `sign`: the preselected certificate's fingerprint. */
  readonly certificate?: string;
  /** `sign`: the digest that was signed, once signed. */
  readonly digest?: Uint8Array<ArrayBuffer>;
  /** The page cancelled it (AbortSignal, or prepare threw). */
  readonly cancelled?: boolean;
}
