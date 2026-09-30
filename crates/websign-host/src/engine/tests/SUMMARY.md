# crates/websign-host/src/engine/tests

- `choose.rs` — Scenario 12 and the `choose` and `status` requests.
- `connection.rs` — Scenarios 1, 2, 9 (disconnect), 14 and 15: the connection itself.
- `listing.rs` — Scenarios 17 to 19: "Still reading {device}", "View in system" and the alternate path's PIN.
- `mod.rs` — The scenarios of `SPEC.md` §8 over fake ports.
- `queue.rs` — Scenarios 11 and 13: one window, ten waiting, device events.
- `rig.rs` — A host process over fake ports, driven with JSON frames.
- `sign.rs` — Scenarios 3 to 6, 9 (cancel) and 10: the happy path, consent, sequence numbers, digest length and deadlines.
- `signing_failures.rs` — Scenarios 7 and 8, and the other outcomes a key store can report.
- `test_signature.rs` — Which signatures mark the setup test done (the project site only).
