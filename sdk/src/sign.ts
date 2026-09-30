import type { SignOptions, SignResult } from "./types";

/**
 * Signs a digest the page prepares for the certificate the person chooses.
 * One confirmation window per call; see {@link SignOptions.prepare}.
 */
export function sign(options: SignOptions): Promise<SignResult> {
  void options;
  throw new Error("unimplemented: SPEC.md §6");
}
