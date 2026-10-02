# crates/websign-host/src/flow/sign

- `deadlines.rs` — The waits a sign request can run out of, besides the person's decision.
- `digest.rs` — The caller answers `sign.need_digest`.
- `failures.rs` — A signature the key store could not make: what the window shows and where the flow goes next.
- `keys.rs` — The key store answers: chains, and signatures verified before they are sent.
- `listed.rs` — A listing arrives.
- `release.rs` — Releasing a certificate to the caller with `sign.need_digest`.
- `signing.rs` — The person presses Sign.
- `ui.rs` — The person acts in the window.
