# app/src/ui

- `confirm/` — the confirmation window
- `diagnostics/` — the diagnostics window
- `theme/` — design tokens and their egui mapping
- `widgets/` — custom widgets egui lacks, one per file
- `bridge.rs` — The [`websign_host::ports::ConfirmUi`] implementation: forwards `UiCommand`s to the UI thread and wakes egui; the window posts `UiEvent`s back into the engine's channel.
- `fonts.rs` — Embedded fonts: Inter 400/500/600 and a JetBrains Mono ASCII subset (OFL-1.1, files in `app/assets/fonts/`), registered as named families because egui does not synthesize weights (`docs/ux.md` §11.2).
- `i18n.rs` — The catalog the windows read their text from: the system locale, unless `WEBSIGN_LOCALE` overrides it.
- `icons.rs` — Phosphor icons (`egui-phosphor`, only the glyphs used are embedded), regular by default, fill for status (`docs/ux.md` §12).
- `mod.rs` — The two windows (egui): confirmation and diagnostics.
- `renderer.rs` — Which renderer draws the windows (evidence: `docs/prototypes/5-ui-screenshots.md`): wgpu, glow re-exec fallback on macOS/Linux.
