# crates/websign-core/tests/present_origin

One test binary; `main.rs` holds the shared helpers.

- `main.rs` — SPEC §9 (`docs/ux.md` §4.3, vectors §16.2): `present::origin::format_origin`.
- `vectors.rs` — the `docs/ux.md` §16.2 vectors, IDN hosts and IP literals shown whole
- `secure_context.rs` — plain `http` only for localhost; other schemes
- `malformed.rs` — text that is not a serialized origin
- `hosts.rs` — IP ranges and Public Suffix List splits
- `robustness.rs` — warning precedence and inputs that must never panic
