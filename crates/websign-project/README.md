# websign-project

Provisional names and identifiers from the repository's
[`project.toml`](../../project.toml) as constants, plus the Chromium
extension ID check. Renaming the product must only require editing
`project.toml` (decision D6) and running `cargo xtask gen`.

`src/generated.rs` is written by `cargo xtask gen` and committed; CI runs
`cargo xtask check generated`. There is no build script, so the crate needs
nothing outside its folder and can be published to crates.io.

- License: **Apache-2.0** (see [`LICENSE`](LICENSE)); used by the Rust client.
