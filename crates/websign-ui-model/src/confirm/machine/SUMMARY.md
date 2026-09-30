# crates/websign-ui-model/src/confirm/machine

- `consent_tests.rs` — Per-certificate consent (D11): only consented certificates skip Continue; the remember box and the two timeouts.
- `edge_tests.rs` — Result screens, the timeout countdown, the repaint deadlines and "View in system".
- `error_tests.rs` — Errors and their recovery, queue and stale commands, the remember box.
- `pin_and_list_tests.rs` — PIN outcomes, the empty list, choose mode, PIN blocks, a token removed while selected.
- `selection_tests.rs` — `is_armed`, `accepts_selection`, moving the selection during a re-arm, and the loading hints.
- `rig.rs` — A scripted clock and shortcuts for driving a `ConfirmModel`.
- `tests.rs` — The main flows: new and remembered callers, arming, focus, selection.
/bin/bash: line 1: unalias: unsetenv: not found
