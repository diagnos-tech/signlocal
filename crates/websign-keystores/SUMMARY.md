# crates/websign-keystores

- `src/` — sources of websign-keystores
- `tests/` — the contract suite against SoftHSM2 and the OS key store, and PKCS#11 integration tests
- `Cargo.toml` — manifest with per-OS dependencies
- `README.md` — what this component is, how to build and test it, where its contract lives
- `SPEC.md` — the behavior contract: rules, errors, edge cases and test vectors (source of truth for blind TDD)
