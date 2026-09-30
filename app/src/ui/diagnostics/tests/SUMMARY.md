# app/src/ui/diagnostics/tests

- `mod.rs` — Kittest checks of the diagnostics window.
- `support.rs` — The harness: the real window over a fake OS and scanner, fixed clock, English, snapshots.
- `fixture.rs` — The mockups' scenario, which is also the input of the golden report (§8.7), and a first run.
- `snapshots.rs` — Every tab (at the top and scrolled to the end), a first run's "Getting started" and the first scan in light and dark.
- `accesskit.rs` — Tab list and lights, opening tab, shortcuts, arrows, row sentences, row actions, FAQ state.
- `golden.rs` — "Copy diagnostics" copies exactly the §8.7 text, from the button and the shortcut; no personal data.
- `revoke.rs` — Revoke → Confirm revoke against a consent store on disk; an unconfirmed revoke expires.
- `drivers.rs` — "Add driver…": a chosen module is remembered and scanned; no picker shows the path field; cancel changes nothing.
- `getting_started.rs` — A first run lists what is missing with one fix each (activate page, driver download, `pcscd` command); a ready computer offers the test page and hides.
- `tree.rs` — The whole AccessKit tree of every tab, mockups and first run: named in words, with bounds (§14).
