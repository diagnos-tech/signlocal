# probe-core/src/verify

Signature verification internals.

- `ec_verify.rs` — ECDSA verification over a precomputed digest
- `rsa_verify.rs` — RSASSA-PKCS1-v1_5 and RSASSA-PSS verification over a precomputed digest
- `tests.rs` — keys that must be refused without panicking
