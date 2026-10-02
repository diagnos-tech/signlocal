# websign-ui-model

What the two windows show and how they react, as pure Rust: certificate rows
(badges, validity, location, hiding, ordering, filter), the confirmation
window's state machine with arming and cancel codes, its contract with the
host (`confirm::port`), and the diagnostics lights, onboarding strip and the
exact "Copy diagnostics" text. No egui, no OS: everything is testable with
injected time.

- UX source: [`docs/ux.md`](../../docs/ux.md) (§4–§8, vectors §16).
- Contract: [`SPEC.md`](SPEC.md).
- License: GPL-3.0-or-later.
