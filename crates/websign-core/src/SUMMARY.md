# crates/websign-core/src

- `cert/` — the tolerant X.509 summary (promoted from probe-core)
- `ecdsa/` — ECDSA signature DER helpers
- `present/` — display rules: origin, holder, document, caller, wire mappings
- `testkit/` — DER builders for unit tests (compiled only in tests)
- `verify/` — per-algorithm signature verification
- `algorithm.rs` — Signature algorithm names, exactly as the SDK and WebCrypto spell them.
- `dedup.rs` — Merging the same certificate seen through more than one key source.
- `ecdsa.rs` — ECDSA curves and conversion between the two signature encodings.
- `fingerprint.rs` — Certificate identity: the SHA-256 of its DER encoding.
- `hash.rs` — Hash algorithms accepted for signing and the digest length rule.
- `hex.rs` — Lowercase hex encoding, the only text form of binary identifiers that the crate needs.
- `lib.rs` — Platform-independent building blocks shared by every key source.
- `pkcs1.rs` — PKCS#1 v1.5 `DigestInfo`, for signers that take an already-hashed input.
- `verify.rs` — Checks a raw signature against a certificate's public key.
