# crates/websign-i18n

- `src/` — sources of websign-i18n
- `tests/` — public-API tests, one file per SPEC section
- `Cargo.toml` — manifest
- `README.md` — what this component is, how to build and test it, where its contract lives
- `SPEC.md` — the behavior contract: rules, errors, edge cases and test vectors (source of truth for blind TDD)
- `build.rs` — Generates `keys.rs` from `i18n/en.toml`, the reference locale: one constant per message, so a misspelled key is a compile error, and the embedded sources of every locale file.
