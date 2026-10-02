# crates/websign-host/src/testing

- `clock.rs` — A clock that only moves when told to.
- `fixture.rs` — One real certificate and a real signature over a fixed digest (test builds only).
- `mod.rs` — Fakes of every port, so a whole host process runs in a test in milliseconds.
- `recorders.rs` — Ports that record what the engine did.
