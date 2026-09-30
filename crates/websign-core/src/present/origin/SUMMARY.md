# crates/websign-core/src/present/origin

- `host.rs` — Host classification: names, IP literals, loopback, eTLD+1.
- `mod.rs` — How a web origin is judged and shown (`docs/ux.md` §4.3, vectors §16.2).
- `parse.rs` — Splitting a serialized origin into scheme, host and port.
