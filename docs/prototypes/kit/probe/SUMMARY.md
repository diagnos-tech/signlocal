# probe

The `websign-probe` binary: lists every signing certificate the machine exposes, signs random digests through every path the app will use, verifies each signature, and works as a native messaging host.

- `Cargo.toml` — crate manifest (platform-specific dependencies for Windows, macOS, and PKCS#11)
- `build.rs` — exposes the identifiers of `project.toml` as compile-time environment variables
- `src/` — the crate's source
