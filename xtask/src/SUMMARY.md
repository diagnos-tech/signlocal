# xtask/src

- `check.rs` — `cargo xtask check [summaries|i18n|generated|release]`.
- `generate.rs` — `cargo xtask gen`.
- `main.rs` — `cargo xtask <command>`: every generation and check step behind one entry point, so CI and people run the same thing (`docs/architecture/testing.md` §CI gates).
- `package.rs` — `cargo xtask package --target <triple>`: release artifacts for one target (`docs/architecture/packaging-and-release.md` §Artifacts).
- `screenshots.rs` — `cargo xtask screenshots --from <dir> --os <name>`: copies e2e PNGs into `docs/screenshots/<os>/` with stable names and rewrites that folder's `SUMMARY.md` and index page (`docs/architecture/testing.md` §Screenshots).
