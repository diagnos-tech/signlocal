# app/src/host_process

- `headless.rs` — A windowless confirmation for e2e builds only (feature `e2e`, `WEBSIGN_E2E_HEADLESS=1`).
- `launcher.rs` — "Open diagnostics" from a connection: a detached `websign diagnostics` process.
- `ui_thread.rs` — The main thread: idle until the first window, then the confirmation window's event loop; fails requests when no window can open.
