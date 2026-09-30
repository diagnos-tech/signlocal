# crates/websign-core/src/present

- `caller.rs` — How a desktop program that asks for a signature is shown and remembered.
- `document.rs` — The holder's document number, masked (`docs/ux.md` §5.5, vectors §16.4).
- `holder.rs` — The holder name a person recognizes (`docs/ux.md` §5.2, vectors §16.3).
- `mod.rs` — Text rules the confirmation window, the diagnostics window and the site all rely on: how an origin, a holder name, a document number and a desktop caller are shown (`docs/ux.md` §4.3, §5.2, §5.5, §16).
- `origin.rs` — How a web origin is judged and shown (`docs/ux.md` §4.3, vectors §16.2).
- `wire.rs` — Conversions between core types and the wire types of `websign-protocol`.
