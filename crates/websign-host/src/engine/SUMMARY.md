# crates/websign-host/src/engine

- `tests/` — the scenarios of `SPEC.md` §8 over fake ports
- `effects.rs` — Doing what a flow asked for, and what follows when it ends.
- `frames.rs` — Frames from the client: accepted by the session, then dispatched by type.
- `lifecycle.rs` — Everything that is not a frame: the window, the key store, devices, deadlines and the end of the connection.
- `listing.rs` — What the engine remembers of the listings: whether one runs ("Still reading {device}") and the last one ("View in system").
- `persist.rs` — Reading and writing the small stores; a failing store never fails a request.
- `present.rs` — What the engine tells a flow before it starts: who asks, and the inputs of the certificate list rules.
- `test_signature.rs` — Marking the setup test done when the project's own site gets a signature.
- `queueing.rs` — Putting requests on the screen, one at a time (`docs/ux.md` §4.11).
- `request.rs` — One open request as the engine holds it.
