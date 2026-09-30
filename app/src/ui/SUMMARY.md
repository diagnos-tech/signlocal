# app/src/ui

- `confirm/` — the confirmation window: every state of `docs/ux.md` §4.8, arming, PIN, snapshots
- `diagnostics/` — the diagnostics window: scan, traffic lights, four tabs, "Copy diagnostics" (also `websign doctor`'s report)
- `theme/` — design tokens and their egui mapping
- `widgets/` — custom widgets egui lacks, one per file
- `audit.rs` — Tests only: a walk over a window's AccessKit tree that flags nodes without a name, without bounds or with an icon glyph in their name (`docs/ux.md` §14).
- `bridge.rs` — The engine's `ConfirmUi`: forwards `UiCommand`s to the UI thread and wakes egui (also when the engine ends); the one window of a process registers its context and native handle here.
- `fonts.rs` — Embedded fonts: Inter 400/500/600 and a JetBrains Mono ASCII subset (OFL-1.1, files in `app/assets/fonts/`), registered as named families because egui does not synthesize weights (`docs/ux.md` §11.2).
- `i18n.rs` — The catalog the windows read their text from: the system locale, unless `WEBSIGN_LOCALE` overrides it.
- `icons.rs` — Phosphor icons (`egui-phosphor`, only the glyphs used are embedded), regular by default, fill for status (`docs/ux.md` §12).
- `mod.rs` — The two windows (egui): confirmation and diagnostics.
- `renderer.rs` — Which renderer draws the windows (evidence: `docs/prototypes/5-ui-screenshots.md`): wgpu, glow re-exec fallback on macOS/Linux.
