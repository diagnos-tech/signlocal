# crates/websign-keystores/tests

- `support/` — shared test code: the SoftHSM2 fixture script and the helper that re-runs a test binary with the token configured
- `contract_os.rs` — the contract against the OS key store, configured by the per-OS CI job through `WEBSIGN_CONTRACT_*` variables (skipped without them)
- `hub_softhsm.rs` — `KeystoreHub` over SoftHSM2: listing, routing by fingerprint and path, sessions surviving a relisting
- `pkcs11_removal.rs` — a SoftHSM2 token deleted while unlocked: the key becomes unavailable and the session ends
- `pkcs11_softhsm.rs` — the contract and the PKCS#11 rules (listing, D5 sessions, PIN state, always-authenticate, chain) against SoftHSM2
