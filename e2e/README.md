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
  `WEBSIGN_E2E_CERTS`, `WEBSIGN_E2E_WINDOW=headless` (no display),
  `WEBSIGN_E2E_HOME` (Linux: the app's folders and log; by default a
  temporary one, removed after the run).
- Before a run: `cargo build -p websign-app --features e2e`,
  `(cd extension && bunx wxt build -b chrome)` (direct channel: the ID is
  `dev_id`), `(cd clients/node && bun run build)`.
- Run: `bun run test` here (Linux: under `xvfb-run -a -s "-screen 0 1600x900x24"`);
  without keys configured, Linux and macOS make a SoftHSM2 token first.
- Screenshots: `cargo xtask screenshots --from <dir> --os <name>`.
- Installed browsers (`browsers/`, `run-browsers.sh`): set `WEBSIGN_E2E_BROWSER_NAME`
  (`chrome`, `edge`, `brave`, `opera`, `vivaldi`, `chromium`, `firefox`) and
  `WEBSIGN_E2E_BROWSER` (its executable) to run the cross-browser core in
  that browser, registered with `websign register --browser <name>` into the
  real location: a private home on Linux, `HKCU` on Windows, the real
  folders on macOS (CI only). Chromium-family browsers get the extension
  through DevTools (`Extensions.loadUnpacked`), Firefox as a temporary
  add-on over WebDriver BiDi (build it with `bunx wxt build -b firefox --mv3`).
  `bash run-browsers.sh <list>` runs every `<label>=<executable>` line of a
  list and writes a results table.
