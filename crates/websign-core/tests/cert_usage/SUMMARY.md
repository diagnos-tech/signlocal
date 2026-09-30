# crates/websign-core/tests/cert_usage

One test binary; `main.rs` holds the shared helpers.

- `main.rs` — SPEC §6 and §6.5: `KeyUsage`, BasicConstraints, EKU, policies and `can_sign`.
- `extensions.rs` — `KeyUsage`, BasicConstraints, EKU and policies as read
- `can_sign.rs` — unknown extensions, `KeyUsage` traits and `can_sign`
