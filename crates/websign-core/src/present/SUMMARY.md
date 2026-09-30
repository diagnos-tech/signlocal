# crates/websign-core/src/present

- `holder/` — display name and title-casing of the holder
- `origin/` — origin parsing, host classification and eTLD+1 split
- `tests/` — unit tests of the text rules over the `docs/ux.md` §16 vectors
- `caller.rs` — How a desktop program that asks for a signature is shown and remembered.
- `document.rs` — The holder's document number, masked (`docs/ux.md` §5.5, vectors §16.4).
- `mod.rs` — Text rules the confirmation window, the diagnostics window and the site all rely on: how an origin, a holder name, a document number and a desktop caller are shown (`docs/ux.md` §4.3, §5.2, §5.5, §16).
- `wire.rs` — Conversions between core types and the wire types of `websign-protocol`.
