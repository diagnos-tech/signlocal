# crates/websign-host/src/session

- `mod.rs` — One connection's protocol state: the `hello` handshake, the negotiated version, the per-transport rules, and which request ids are open.
- `steps.rs` — The individual checks of `Session::accept`, one function per rule of `SPEC.md` §2.
- `tests.rs` — Session rules of `SPEC.md` §2, driven with hand-written frames.
- `transport.rs` — The two transports and what each guarantees about the caller.
- `validate.rs` — Transport-specific rules on top of strict parsing (`SPEC.md` §2.2).
