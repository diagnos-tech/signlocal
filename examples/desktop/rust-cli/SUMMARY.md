# examples/desktop/rust-cli

- `src/` — the CLI: SHA-256 of a file, signed through the fake or the real app
- `tests/` — runs the built CLI against the fake app
- `Cargo.toml` — standalone workspace depending on `websign-client` by path
- `README.md` — how to run the example and what its exit codes mean
