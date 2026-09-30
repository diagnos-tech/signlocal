# crates/websign-keystores/src/windows

- `acquire.rs` — Opening a certificate's private key with `CryptAcquireCertificatePrivateKey`, the one call that reaches CNG providers, legacy CSPs and minidrivers alike.
- `capi.rs` — Signing with a legacy CAPI key (`CryptSignHash`): old token CSPs and A1 certificates imported into Microsoft's software CSPs.
- `errors.rs` — Windows status codes turned into [`KeystoreError`], so the app can tell a cancelled PIN dialog or a wrong PIN apart from a real failure.
- `handles.rs` — Native key handles, each released exactly once by its owner.
- `hardware.rs` — Whether a provider keeps its keys in hardware, asked of the provider itself (never of a key), so that listing never touches a card.
- `key_info.rs` — Where a certificate's private key lives, read from the certificate's `CERT_KEY_PROV_INFO` property only, so that listing never opens a key, never touches a card and never prompts.
- `mod.rs` — Windows certificate store (`CurrentUser\MY`), signing through CNG (NCrypt) or legacy CAPI, whichever the key's provider speaks.
- `ncrypt.rs` — Signing with a CNG key (`NCryptSignHash`).
- `store.rs` — `CurrentUser\MY` and the certificates in it.
- `thumbprint.rs` — The SHA-1 thumbprint Windows indexes certificates by.
- `window.rs` — Which window owns the OS PIN dialog.
