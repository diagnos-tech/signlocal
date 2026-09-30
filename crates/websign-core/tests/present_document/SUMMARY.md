# crates/websign-core/tests/present_document

One test binary; `main.rs` holds the shared helpers.

- `main.rs` — SPEC §11 (`docs/ux.md` §5.5, vectors §16.4): `present::document`.
- `rules.rs` — the §16.4 vectors, priority and CPF shapes
- `etsi.rs` — ETSI EN 319 412-1 national identifiers
- `certificates.rs` — labels of the OpenSSL fixtures
- `privacy.rs` — the full CPF and national identifier never leak
