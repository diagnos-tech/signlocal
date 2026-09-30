# crates/websign-core/tests/verify_brainpool

One test binary; `main.rs` holds the shared helpers.

- `main.rs` — SPEC §4.1 and §7: `verify` on Brainpool keys. The signatures were made by
- `valid.rs` — signatures that verify
- `invalid.rs` — signatures that must not verify
- `check_order.rs` — which error wins, and unusable keys
