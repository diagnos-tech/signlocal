# crates/websign-core/tests/ecdsa_brainpool

One test binary; `main.rs` holds the shared helpers.

- `main.rs` — SPEC §4 and §4.1: `Curve` with the Brainpool curves, OID mapping, and the
- `curve.rs` — `Curve` properties and `from_oid`
- `encodings.rs` — DER and raw encodings on the Brainpool field sizes
- `openssl.rs` — the OpenSSL Brainpool vectors
