# crates/websign-host/src/runtime/softhsm_tests

- `child.rs` — The part of the SoftHSM2 run that sees the token: a client on pipes talks to `serve` with the real key worker.
- `mod.rs` — The parent test: builds the token and re-runs this binary with it.
