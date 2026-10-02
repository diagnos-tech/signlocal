# crates/websign-core/src/verify

- `ec_verify.rs` — ECDSA verification over a digest that is already computed.
- `rsa_verify.rs` — RSA signature verification over a digest that is already computed.
- `tests.rs` — Keys that the `rsa` and `p256`/`p384`/`p521` crates must refuse, reported as `UnsupportedKey` and never as a panic.
