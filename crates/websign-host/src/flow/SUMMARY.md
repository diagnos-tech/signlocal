# crates/websign-host/src/flow

- `choose/` — building the `choose.result`: which certificates, then their chains
- `sign/` — the sign flow, one file per transition group
- `certificate.rs` — A candidate as the caller receives it.
- `choose.rs` — The `choose` flow: remembered caller → its certificates without a window; otherwise the window in choose mode (`docs/plan.md` D2).
- `context.rs` — What the certificate list depends on besides the candidates.
- `errors.rs` — Final error replies and how the window is told about them.
- `listing.rs` — A listing as a flow sees it: the candidates, the list the window will build from them, and which certificates may be chosen.
- `mod.rs` — Per-request state machines.
- `presentation.rs` — What the window needs to show about who asks and where the request stands.
- `sign.rs` — The `sign.begin` flow (`docs/architecture/protocol.md` §Sign flow).
