# probe-core

Platform-independent building blocks shared by every key source: algorithms, signature encodings, certificate summaries, verification, and deduplication. Nothing here talks to the OS.

- `Cargo.toml` — crate manifest
- `SPEC.md` — the specification the tests and the code were written from (blind TDD)
- `src/` — the implementation and its unit tests
- `tests/` — integration tests, one file per specification section, over OpenSSL-made fixtures
