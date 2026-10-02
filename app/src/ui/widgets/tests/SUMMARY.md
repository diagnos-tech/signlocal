# app/src/ui/widgets/tests

- `mod.rs` — Kittest checks of every widget: AccessKit roles, names and states, and light/dark snapshots.
- `support.rs` — The harness: our fonts, a pinned theme, the canvas, per-OS snapshot baselines.
- `button.rs` — Arming, busy, click, Space and Enter behavior of buttons; snapshots.
- `cert_row.rs` — Radio semantics, full accessible names, disabled rows, Enter never selects; snapshots.
- `code_card.rs` — The spoken code and the decorative identicon; snapshots.
- `fields.rs` — PIN typed into the buffer, no value in AccessKit, paste refused, show/hide; filter field; snapshots.
- `loading.rs` — Spinner role and name; spinner and skeleton snapshots.
- `notices.rs` — Chip long names, banners as live regions; snapshots.
- `specimen.rs` — Every text token and icon, so a lost glyph shows in the snapshot.
- `tabs.rs` — Tab role, selection and status in the name; snapshots.
