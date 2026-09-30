# crates/websign-host/tests/common

- `mod.rs` — re-exports and the two shared origins
- `certs.rs` — fixture certificates, digests and real signatures from websign-core
- `effects.rs` — readers for the `Effect`s a flow returns
- `fakes.rs` — recording fakes for the ports and a manual clock
- `flows.rs` — sign flows built in a given state
- `harness.rs` — an engine wired to fakes with one-line scenario steps
- `out.rs` — what one step produced, with readers per port
- `stores.rs` — in-memory stores with SPEC §7 semantics
- `wire.rs` — client frames as JSON
