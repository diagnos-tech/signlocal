# crates/websign-host/src/runtime/softhsm_tests

- `alternate.rs` — "Try through the token driver": a failing OS store with SoftHSM2 as the alternate path; the driver gets our PIN.
- `child.rs` — The part of the SoftHSM2 run that sees the token: a client on pipes talks to `serve` with the real key worker.
- `model_window.rs` — A scripted window made of the real `ConfirmModel`, pressing what a person would.
- `mod.rs` — The parent test: builds the token and re-runs this binary with it, once per child module.
