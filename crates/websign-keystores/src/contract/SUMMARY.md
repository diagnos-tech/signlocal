# crates/websign-keystores/src/contract

- `listing.rs` — `lists-expected`, `list-is-quiet` (also: listing never unlocks a token), `provider-is-anonymous`
- `pin.rs` — `wrong-pin`, `session-reuse` (D5) and `always-authenticate`, for keys whose PIN the app collects
- `signing.rs` — `signs-every-combination` (verified with `websign_core::verify`), `rejects-wrong-length`, `vanished-key`, `chain-best-effort`
- `support.rs` — helpers shared by the checks: signing a fixed message, which PIN a key needs, naming keys without personal data
