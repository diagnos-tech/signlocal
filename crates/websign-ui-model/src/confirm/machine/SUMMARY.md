# crates/websign-ui-model/src/confirm/machine

- `edge_tests.rs` — Result screens, the timeout countdown and the repaint deadlines.
- `error_tests.rs` — Errors and their recovery, queue and stale commands, the remember box.
- `pin_and_list_tests.rs` — PIN outcomes, the empty list, choose mode, PIN blocks, a token removed while selected.
- `rig.rs` — A scripted clock and shortcuts for driving a `ConfirmModel`.
- `tests.rs` — The main flows: new and remembered callers, arming, focus, selection.
