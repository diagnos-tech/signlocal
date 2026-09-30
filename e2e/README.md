# e2e — end-to-end tests

Playwright suites that drive a fixture page through the SDK, the unpacked
extension in Chromium, and the app built with `--features e2e` (it confirms by
itself after the real arming delay and saves a PNG of every window state),
signing with software keys and verifying every signature.

- Plan, scenarios, matrix and screenshots: [`docs/architecture/testing.md`](../docs/architecture/testing.md) §5.
- Environment: `WEBSIGN_E2E_APP`, `WEBSIGN_E2E_EXTENSION`,
  `WEBSIGN_E2E_SCREENSHOTS`, `WEBSIGN_E2E_BROWSER` (`lib/environment.ts`).
- Run: `bun run test` in this folder.
