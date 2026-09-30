# e2e — end-to-end tests

Playwright suites that drive a page through the SDK, the unpacked extension
in Chromium, and the app built with `--features e2e`, signing with software
keys and verifying every signature with `node:crypto`.

- Plan, scenarios, matrix and screenshots: [`docs/architecture/testing.md`](../docs/architecture/testing.md) §5.
- The app's e2e hooks (`app/src/e2e.rs`): the real confirmation window acts
  by itself through accessibility actions once armed
  (`WEBSIGN_E2E_CONFIRM=sign|choose|remember|cancel`), types
  `WEBSIGN_E2E_PIN`, and saves every state it shows, light and dark, to
  `WEBSIGN_E2E_SCREENSHOTS` as `<window>-<state>-<theme>.png`.
- Environment (`lib/environment.ts`, which also says how each OS gets its
  keys): `WEBSIGN_E2E_APP` (required), `WEBSIGN_E2E_EXTENSION`,
  `WEBSIGN_E2E_SCREENSHOTS`, `WEBSIGN_E2E_BROWSER`, `WEBSIGN_E2E_SOFTHSM` or
  `WEBSIGN_E2E_CERTS`, `WEBSIGN_E2E_WINDOW=headless` (no display).
- Before a run: `cargo build -p websign-app --features e2e`,
  `(cd extension && bunx wxt build -b chrome)` (direct channel: the ID is
  `dev_id`), `(cd clients/node && bun run build)`.
- Run: `bun run test` here (Linux: under `xvfb-run -a -s "-screen 0 1600x900x24"`);
  without keys configured, Linux and macOS make a SoftHSM2 token first.
- Screenshots: `cargo xtask screenshots --from <dir> --os <name>`.
