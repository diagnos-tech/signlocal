import type { Certificate, CertificateOptions } from "./types";

/**
 * The certificate the person chooses in the app's window — never the list of
 * the computer. A remembered site gets the certificates it already used,
 * without a window.
 */
export function certificates(options?: CertificateOptions): Promise<Certificate[]> {
  void options;
  throw new Error("unimplemented: SPEC.md §5");
}
