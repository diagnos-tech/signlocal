# app — the `websign` binary

One executable, started four ways (`src/main.rs`): by a browser as the native
messaging host, by the OS for `websign:` URLs, by programs and people through
the CLI (`sign`, `choose`, `connect`, `install`, `doctor`, …), and from the app
menu to open diagnostics. It hosts the confirmation and diagnostics windows
(egui, wgpu) and the OS glue that is not a key store.

- CLI contract: [`docs/architecture/desktop-api.md`](../docs/architecture/desktop-api.md).
- Processes and threads: [`docs/architecture/overview.md`](../docs/architecture/overview.md).
- UI: [`docs/ux.md`](../docs/ux.md); pure view logic lives in
  [`crates/websign-ui-model`](../crates/websign-ui-model/).
- Build: `cargo build -p websign-app --release` (binary `websign`). Test
  builds for e2e: `--features e2e` (never released; `tests/release_build.rs`
  checks the default binary has no e2e hook).
- Tests: `cargo test -p websign-app` (CLI contract, launch shapes, log
  privacy audit); `cargo test -p websign-app --features e2e` adds
  `websign connect` signing with a SoftHSM2 token through `websign-client`
  (skipped without SoftHSM2 unless `WEBSIGN_REQUIRE_SOFTHSM=1`).
- Logs: the per-user log folder (`~/.local/state/websign`,
  `~/Library/Logs/websign`, `%LOCALAPPDATA%\websign\logs`), level from
  `WEBSIGN_LOG`, every line privacy-filtered.
- License: GPL-3.0-or-later with the app-store permission.
