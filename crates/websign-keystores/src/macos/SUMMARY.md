# crates/websign-keystores/src/macos

- `algorithm.rs` — `SecKeyAlgorithm` for each (hash, signature algorithm) pair.
- `errors.rs` — Security.framework and CryptoTokenKit failures turned into [`KeystoreError`], so the app can tell a dismissed PIN dialog or a wrong PIN apart from a real failure.
- `identities.rs` — Finding identities (certificate plus private key) without prompting.
- `mod.rs` — MacOS keychains, including CryptoTokenKit smart card tokens, signing through `SecKeyCreateSignature`.
- `sign.rs` — Signing a digest with an identity's private key.
- `token.rs` — Which CryptoTokenKit token, if any, holds a private key.
