# probe/src/keystores/windows

Windows certificate store (`CurrentUser\MY`), signing through CNG or legacy CAPI according to the key's provider.

- `acquire.rs` — opens a private key with `CryptAcquireCertificatePrivateKey`
- `capi.rs` — signing with a legacy CAPI key, including reopening A1 containers in the AES CSP
- `errors.rs` — Windows status codes mapped to typed keystore errors
- `handles.rs` — native key handles, each released exactly once by its owner
- `hardware.rs` — whether a provider keeps keys in hardware, asked of the provider, never of a key
- `key_info.rs` — where a certificate's key lives, read from `CERT_KEY_PROV_INFO` only, without opening the key
- `mod.rs` — the Windows key source
- `ncrypt.rs` — signing with a CNG key (`NCryptSignHash`)
- `store.rs` — the current user's personal store, opened read-only
- `thumbprint.rs` — the SHA-1 thumbprint Windows indexes certificates by, used only to find a certificate again
- `window.rs` — which window owns the OS PIN dialog
