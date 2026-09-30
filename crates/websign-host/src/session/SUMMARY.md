# crates/websign-host/src/session

- `mod.rs` — One connection's protocol state: the `hello` handshake, the negotiated version, the per-transport rules, and which request ids are open.
- `transport.rs` — The two transports and what each guarantees about the caller.
- `validate.rs` — Transport-specific rules on top of strict parsing (`SPEC.md` §2.2).
