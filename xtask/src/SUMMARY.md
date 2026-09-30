# xtask/src

- `check/` — `cargo xtask check`: the repository invariants
- `generate/` — `cargo xtask gen`: one generator per derived file family
- `screenshots/` — `cargo xtask screenshots`: collects e2e PNGs into docs/screenshots
- `fsutil.rs` — file helpers whose errors name the path and the action
- `i18n_files.rs` — reads `i18n/<locale>.toml` for generation and checks
- `main.rs` — command-line entry point: `gen`, `check`, `package`, `screenshots`
- `package.rs` — `cargo xtask package`: release artifacts for one target, SHA256SUMS
- `package/` — one module per artifact kind (deb, rpm, tar.gz, zip, app, extension) and helpers
- `plan.rs` — the set of files a generation run wants; written by `gen`, compared by `check generated`
- `root.rs` — finds the repository root from any folder
- `testutil.rs` — temporary folders for unit tests
