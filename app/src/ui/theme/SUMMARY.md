# app/src/ui/theme

- `mod.rs` — The design tokens of `docs/ux.md` §11 and their mapping onto egui `Visuals`/`Style` (§11.5); installs fonts and both themes following the OS from eframe's app creator and returns the `Installed` token windows require.
- `tokens.rs` — Color tokens (§11.1) for light and dark, and the identicon palette.
- `typography.rs` — Type tokens (§11.2) and their egui `TextStyle`s.
- `metrics.rs` — Spacing, sizes and radii (§11.3).
- `motion.rs` — Motion tokens (§11.4) and the "reduce motion" switch.
- `visuals.rs` — The egui `Style` built from the tokens (§11.5).
- `css_expected.rs` — Test-only: the CSS text every token must have, built from the Rust constants.
- `css_parse.rs` — Test-only reader for the CSS subset of `design/tokens.css`.
- `css_tests.rs` — Tests that every `--ws-*` token of `design/tokens.css` equals its Rust constant.
- `contrast_tests.rs` — Tests that the color pairs meet WCAG AA in both themes (§11.1, §14).
