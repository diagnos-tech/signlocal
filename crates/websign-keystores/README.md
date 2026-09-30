# websign-keystores

Every source of signing keys behind one `Keystore` trait: the Windows
certificate store (CNG and legacy CAPI), the macOS keychain and CryptoTokenKit
tokens, and one keystore per PKCS#11 module on every OS. `KeystoreHub` opens
them once per process, caches the de-duplicated listing and routes
signatures; `contract::run` is the suite every adapter must pass.

- Adapters, model and inventory are promoted from the Phase-0 kit, where CI
  proved them with software keys (see `docs/prototypes/1-windows.md`,
  `2-mac.md`, `4-linux.md`).
- Contract: [`SPEC.md`](SPEC.md) — sessions (D5), D9 fallback, device links,
  chains, hub.
- Tests: `cargo test -p websign-keystores`; contract runs in per-OS CI jobs
  with SoftHSM2, software KSPs and a temporary keychain.
- License: GPL-3.0-or-later.
