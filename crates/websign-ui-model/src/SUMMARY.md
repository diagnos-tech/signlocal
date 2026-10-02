# crates/websign-ui-model/src

- `certs/` — the certificate list rules
- `confirm/` — the confirmation window model and its host contract
- `diagnostics/` — diagnostics lights, onboarding and the report text
- `fixtures.rs` — Test candidates and contexts shared by the unit tests.
- `lib.rs` — What the windows show and how they react, as pure functions and state machines (`docs/ux.md` §4–§8, test vectors §16).
- `possible.rs` — "Possible certificates" as the window shows them (`docs/ux.md` §6.2).
- `time.rs` — Relative times for diagnostics ("3 min ago", "yesterday, 17:40").
