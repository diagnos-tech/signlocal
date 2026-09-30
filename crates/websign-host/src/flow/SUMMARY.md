# crates/websign-host/src/flow

- `choose.rs` — The `choose` flow: remembered caller → its certificates at once, no window; otherwise the window in choose mode (`docs/plan.md` D2).
- `mod.rs` — Per-request state machines.
- `sign.rs` — The `sign.begin` flow (`docs/architecture/protocol.md` §Sign flow).
