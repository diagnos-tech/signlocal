# app/src/ui/diagnostics

- `facts/` — one scan of this computer (browsers, devices, drivers, certificates), off the UI thread
- `view/` — drawing the window: sidebar, header, "Getting started" and the four tabs; views return actions
- `tests/` — kittest checks: snapshots of every tab, AccessKit and keyboard, golden report, revoke flow
- `snapshots/` — reviewed kittest baselines, one folder per OS
- `mod.rs` — The diagnostics window (`docs/ux.md` §8), 760 × 540: `run` opens it with eframe.
- `window.rs` — The window's state and frame: collect the scan, read shortcuts, draw, apply actions.
- `apply.rs` — Carrying out what the person did: consent and settings stores, OS actions, rescans.
- `state.rs` — Window state between frames (tab, confirm-revoke timer, notices) and the `Action`s views emit.
- `services.rs` — OS actions (links, viewer, `.pfx`, repair) and the thread scanner, behind traits for tests.
- `pick.rs` — The OS file picker for "Add driver…" and the macOS `.pfx` import; answers arrive by channel.
- `live.rs` — PC/SC device events that trigger a new scan while the window is open (§8.4).
- `keys.rs` — Window shortcuts: Ctrl/⌘+1…4, F5 / Ctrl/⌘+R, Ctrl/⌘+Shift+C, Ctrl/⌘+W (§8.8).
- `lights.rs` — Traffic lights per tab and overall, the opening tab, "Getting started" (via `websign-ui-model`).
- `remembered.rs` — Remembered sites and programs from the consent store, named as the person saw them.
- `report.rs` — The "Copy diagnostics" text (§8.7) from a scan; also `websign doctor`'s report input.
- `counts.rs` — Certificate counts for the report: numbers and categories only.
