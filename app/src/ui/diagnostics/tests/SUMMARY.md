# app/src/ui/diagnostics/tests

- `mod.rs` — Kittest checks of the diagnostics window.
- `support.rs` — The harness: the real window over a fake OS and scanner, fixed clock, English, snapshots.
- `fixture.rs` — The mockups' scenario, which is also the input of the golden report (§8.7).
- `snapshots.rs` — Every tab (at the top and scrolled to the end) and the first scan in light and dark.
- `accesskit.rs` — Tab list and lights, opening tab, shortcuts, arrows, row sentences, row actions, FAQ state.
- `golden.rs` — "Copy diagnostics" copies exactly the §8.7 text, from the button and the shortcut; no personal data.
- `revoke.rs` — Revoke → Confirm revoke against a consent store on disk; an unconfirmed revoke expires.
- `drivers.rs` — "Add driver…": a chosen module is remembered and scanned; no picker shows the path field; cancel changes nothing.
