# probe/src/keystores/macos

macOS keychains and CryptoTokenKit tokens, signing through `SecKeyCreateSignature`.

- `algorithm.rs` — the `SecKeyAlgorithm` for each (hash, signature algorithm) pair, digest variants only
- `errors.rs` — Security.framework and CryptoTokenKit failures mapped to typed keystore errors
- `identities.rs` — finds identities without prompting, with a separate query for token identities
- `mod.rs` — the two sources, `macos:keychain` and `macos:ctk`
- `sign.rs` — signs a digest with an identity's private key; converts ECDSA DER to raw `r || s`
- `token.rs` — which CryptoTokenKit token, if any, holds a private key
