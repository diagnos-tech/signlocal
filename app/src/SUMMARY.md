# app/src

- `cli/` — the command line: desktop API and installer entry points
- `platform/` — OS facts and actions outside key stores and registration
- `ui/` — the egui windows: rendering only, decisions come from websign-ui-model
- `e2e.rs` — Test-only behavior (feature `e2e`, never in release builds; CI fails the release job if the release binary contains the marker string `WEBSIGN_E2E_BUILD`).
- `host_process/` — the threads of a host process: UI thread, diagnostics launcher, e2e headless confirmation
- `host_process.rs` — A host process: the engine on a worker thread, the confirmation window on the main thread, started only when needed.
- `launch/` — `websign:` URL parsing and the argv-shape tests
- `launch.rs` — Telling apart the ways this binary is started: browser (our extensions only), `websign:` URL, `connect`, no arguments, command line.
- `logging/` — log folder, rotating file, privacy filter and its audit tests
- `logging.rs` — The app's log: a size-capped, privacy-filtered file in the per-user log folder, because browsers discard a native host's stderr.
- `main.rs` — `websign`: the WebeSign desktop app.
