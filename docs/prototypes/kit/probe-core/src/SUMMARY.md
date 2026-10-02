# probe-core/src

Implementation of the specification.

- `algorithm.rs` — signature algorithm names as the SDK spells them
- `cert/` — tolerant summary of an X.509 certificate
- `dedup.rs` — merging the same certificate seen through more than one key source
- `ecdsa/` — ECDSA signature DER primitives
- `ecdsa.rs` — ECDSA curves and DER to raw `r || s` conversion
- `fingerprint.rs` — certificate identity: SHA-256 of the DER
- `hash.rs` — accepted hash algorithms and the digest length rule
- `hex.rs` — lowercase hex encoding
- `lib.rs` — crate overview and public exports
- `pkcs1.rs` — PKCS#1 v1.5 `DigestInfo` for signers that take an already-hashed input
- `testkit/` — hand-assembled DER certificates for unit tests
- `verify/` — RSA and ECDSA verification internals
- `verify.rs` — checks a raw signature against a certificate's public key
