// The samples of the "Quickstart" part of the developers page (site/). The page cuts the
// regions below out of this file, so they are type-checked with the rest of examples/web.

declare function signedAttributesFor(certificate: {
  readonly der: Uint8Array;
}): Uint8Array<ArrayBuffer>;

// #region site:quickstart
import { sign } from "@signlocal/sdk";

const { signature, certificate } = await sign({
  hash: "SHA-256",
  prepare: (certificate, { hash }) => crypto.subtle.digest(hash, signedAttributesFor(certificate)),
});

// #endregion site:quickstart

export { certificate, signature };
