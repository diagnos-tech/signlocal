# ui-probe

Spike proving the real app window can be rendered and screenshotted on every CI target.

- `Cargo.toml`: crate manifest; eframe with both `wgpu` and `glow` compiled in.
- `src/renderer.rs`: `auto|wgpu|glow` choice and the wgpu-then-glow fallback (to be promoted).
- `src/app.rs`: eframe app; renders warm-up frames, requests a screenshot, closes.
- `src/ui.rs`: the sample UI (embedded font, button, list) shared by window and headless paths.
- `src/screenshot.rs`: PNG writer.
- `src/main.rs`: CLI (`--renderer`, `--out`, `--timeout-secs`) printing a `RESULT` line.
- `tests/snapshot.rs`: headless egui_kittest render with wgpu; prints the adapter it got.
- `ci/run-spike.sh`: runs every approach for one OS, records a result table, never fails.
- `assets/`: embedded DejaVu Sans (Bitstream Vera license, free to redistribute).
