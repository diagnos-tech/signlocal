# crates/websign-core/tests/ecdsa

One test binary; `main.rs` holds the shared helpers.

- `main.rs` — SPEC §4: `Curve`, `der_to_raw`, `raw_to_der`.
- `curve.rs` — Curve
- `der_to_raw_valid.rs` — der_to_raw: valid input
- `der_to_raw_rejected.rs` — der_to_raw: rejected input
- `raw_to_der.rs` — raw_to_der
- `round_trip.rs` — round trip and OpenSSL vectors
