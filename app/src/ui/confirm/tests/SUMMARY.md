# app/src/ui/confirm/tests

- `mod.rs` — Kittest checks of the confirmation window.
- `support.rs` — The harness: the real window at 480 × 600, a manual clock, the engine's end of the channel, per-OS snapshots.
- `fixtures.rs` — The mockups' people, certificates, callers and code.
- `scenes.rs` — Every state of §4.8 as a named script of host commands and waits.
- `snapshots.rs` — One snapshot per scene in light and dark.
- `arming.rs` — 600 ms of focus before Sign or Continue act; early clicks and keystrokes dropped (§4.7).
- `keyboard.rs` — Esc cancels, Enter never continues or chooses, Space does, arrows select, Enter on a row moves focus (§4.9).
- `callers.rs` — An interpreter reads "A script run by {program}" and cannot be remembered; a digest timeout blames the site (§4.3.1, §4.11).
- `engine_end.rs` — The window closes when the engine ends, also while hidden (logic-only passes), after any notice's hold.
- `accesskit.rs` — Header sentence, radio rows with full names, PIN without value, alerts, Tab order (§14).
- `open_time.rs` — Frames laid out within 300 ms of `Open` (timing printed).
- `pin_fit.rs` — Our PIN field stays visible above the footer, the body unscrolled, under a long list at 480 × 600 (§4.6).
- `paths.rs` — "View in system" asks the host for the selected certificate; "Try through the token driver" shows the driver's PIN field first (§5.11, §5.12).
