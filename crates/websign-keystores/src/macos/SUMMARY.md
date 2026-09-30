# crates/websign-keystores/src/macos

- `errors/` — unit tests of the `CFError` mapping
- `algorithm.rs` — `SecKeyAlgorithm` for each (hash, signature algorithm) pair, "Digest" variants only.
- `chain.rs` — Issuer certificates of a leaf from `SecTrust`, without network fetching.
- `errors.rs` — Security.framework and CryptoTokenKit `CFError`s turned into [`KeystoreError`] (cancel, wrong/blocked PIN, token removed).
- `identities.rs` — Finding identities (certificate plus private key) in keychain files and CryptoTokenKit tokens without prompting.
- `mod.rs` — The `macos:keychain` and `macos:ctk` key sources, signing through `SecKeyCreateSignature`.
- `serial.rs` — One process-wide lock around every Security.framework call.
- `sign.rs` — Signing a digest with an identity's private key; ECDSA converted to raw `r || s`.
- `status.rs` — `OSStatus` results turned into [`KeystoreError`], with symbolic names.
- `token.rs` — Which CryptoTokenKit token, if any, holds a private key, and its driver name.
- `user_info.rs` — PIN attempts left, read from a `CFError`'s `userInfo`.
