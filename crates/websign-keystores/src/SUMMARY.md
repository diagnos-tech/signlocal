# crates/websign-keystores/src

- `contract/` — the checks of the contract suite, one file per group (listing, signing, PIN and sessions)
- `hub/` — unit tests of the hub, run against fake key sources
- `macos/` — macOS keychain and CryptoTokenKit (promoted from the kit)
- `pkcs11/` — PKCS#11 modules: discovery, listing without a PIN, D5 sessions, signing, PIN state, device links, chains
- `windows/` — Windows certificate store via CNG and CAPI (promoted from the kit)
- `capabilities.rs` — `KeyCapabilities`: the signature algorithms a key store can produce with a key, read without prompting.
- `contract.rs` — The contract every [`Keystore`] adapter must satisfy, as runnable checks (`contract::run`).
- `fake.rs` — A scripted key source for unit tests of the hub (test builds only).
- `hub.rs` — The host's single entry point to every key source: lazy open, cached listing, routing by fingerprint and path.
- `inventory.rs` — Everything every key source can see, parsed and grouped by certificate.
- `lib.rs` — Key sources: every place a signing certificate can come from.
- `model.rs` — Data exchanged between the commands and the key sources.
