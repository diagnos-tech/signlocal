import type { VerificationCode } from "./types";

/**
 * The verification code of `digest`, identical to the one the app shows
 * (same algorithm as websign-protocol `verification_code`). Show it next to
 * your own Sign button so people can compare.
 */
export function fingerprint(digest: Uint8Array): VerificationCode {
  void digest;
  throw new Error("unimplemented: SPEC.md §9");
}
