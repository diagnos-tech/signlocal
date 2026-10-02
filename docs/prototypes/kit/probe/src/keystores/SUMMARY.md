# probe/src/keystores

Key sources: every place a signing certificate can come from. Each implements the `Keystore` trait and never interprets certificates.

- `inventory.rs` — everything every source can see, parsed and grouped by certificate; read by the CLI and the native host
- `macos/` — macOS Keychain and CryptoTokenKit through Security.framework
- `mod.rs` — the `Keystore` trait and the per-OS module wiring
- `model.rs` — data exchanged between commands and key sources
- `pkcs11/` — PKCS#11 modules: p11-kit registrations, known vendor paths, and explicit modules
- `windows/` — Windows certificate store, signing through CNG or legacy CAPI
