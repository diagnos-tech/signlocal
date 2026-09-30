# crates/websign-keystores/src

- `macos/` — macOS keychain and CryptoTokenKit (promoted from the kit)
- `pkcs11/` — PKCS#11 modules (promoted from the kit)
- `windows/` — Windows certificate store via CNG and CAPI (promoted from the kit)
- `contract.rs` — The contract every [`Keystore`] adapter must satisfy, as runnable checks.
- `hub.rs` — The host's single entry point to every key source.
- `inventory.rs` — Everything every key source can see, parsed and grouped by certificate.
- `lib.rs` — Key sources: every place a signing certificate can come from.
- `model.rs` — Data exchanged between the commands and the key sources.
