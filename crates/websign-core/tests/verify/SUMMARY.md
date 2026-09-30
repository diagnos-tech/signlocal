# crates/websign-core/tests/verify

One test binary; `main.rs` holds the shared helpers.

- `main.rs` — SPEC §7: `verify`, `VerifyError`.
- `valid.rs` — valid signatures: every hash × algorithm × key
- `invalid_1.rs` — invalid signatures (part 1 of 3)
- `invalid_2.rs` — invalid signatures (part 2 of 3)
- `invalid_3.rs` — invalid signatures (part 3 of 3)
- `error_order.rs` — error order
- `verify_error.rs` — `VerifyError`
