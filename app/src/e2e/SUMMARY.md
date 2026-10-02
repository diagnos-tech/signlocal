# app/src/e2e

- `capture.rs` — Saving what a window shows, light then dark, through `ViewportCommand::Screenshot`.
- `confirm_driver.rs` — The confirmation window's driver: screenshots of each state and the person's part as accessibility actions.
- `diagnostics_driver.rs` — The diagnostics window's driver: save the settled tab in both themes, then close.
- `png.rs` — Screenshots as PNG files.
- `shadow.rs` — A copy of the window's model fed the same host commands, naming the state on screen.
- `tree.rs` — The window's widgets as its accessibility tree lists them, and accessibility actions on them.
