# app/src/ui/confirm

- `mod.rs` — The confirmation window (`docs/ux.md` §4–§6), 480 × 600, fixed: module map and `run`, the entry point of a host process's UI thread.
- `run.rs` — The eframe event loop: host commands between frames (also while hidden), the close button as Cancel, closing once the engine is done, also while hidden.
- `window.rs` — The window's state and frame: commands in, person input through the model, decisions out; no eframe, so kittest drives it as the event loop does.
- `session.rs` — Per-request window state the model does not keep: the PIN buffer, what is expanded, pending focus.
- `guard.rs` — Drops (and wipes) keystrokes that arrive before the window is armed, except Esc and, once the model takes selections, ↑/↓/Home/End (§4.7).
- `outbox.rs` — Turns the model's intents into `UiEvent`s; the PIN goes into a `SecretString` and the field is wiped.
- `viewport.rs` — The OS window: size, always on top, show/hide, title, raising through `platform::focus`, the native handle.
- `clock.rs` — The time source: the system clock, or a hand-moved one for tests.
- `words.rs` — Small localized pieces: the site, the window title, browser/hash/algorithm names, the spelled-out code.
- `row_text.rs` — The localized texts of a certificate row and its sentence for screen readers (§5, §14).
- `failure_text.rs` — Title, text and technical detail of each failure's notice (§15).
- `view/` — Drawing one `ConfirmView`: header, body per state, footer
- `tests/` — Kittest checks: snapshots of every state, arming, keys, AccessKit, time to first frame
- `snapshots/` — reviewed kittest baselines, one folder per OS
