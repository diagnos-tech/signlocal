# probe/src/nm/keystore_backend

Parts and tests of the real native messaging backend.

- `fixture.rs` — a self-signed ECDSA P-256 certificate and a signature made with its key, for tests without a token
- `sources.rs` — the machine's key sources, opened and listed once per process
- `tests.rs` — unit tests using fake key sources
