# app/src

- `cli/` — the command line: desktop API and installer entry points
- `platform/` — OS facts and actions outside key stores and registration
- `ui/` — the egui windows: rendering only, decisions come from websign-ui-model
- `e2e.rs` — Test-only behavior (feature `e2e`, never in release builds; CI fails the release job if the release binary contains the marker string `WEBSIGN_E2E_BUILD`).
- `host_process.rs` — A host process: the engine on a worker thread, the UI on the main thread.
- `launch.rs` — Telling apart the ways this binary is started (see `main.rs`).
- `logging.rs` — The app's log: a size-capped file in the temp folder, because browsers discard a native host's stderr.
- `main.rs` — `websign`: the WebeSign desktop app.
