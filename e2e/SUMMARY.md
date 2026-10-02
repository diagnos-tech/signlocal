# e2e

- `audit/` — checks that run after every scenario (the log audit)
- `browsers/` — the cross-browser core, run once per installed browser (WEBSIGN_E2E_BROWSER_NAME)
- `fixtures/` — pages the e2e tests load
- `lib/` — e2e helpers: environment, keys, browser, app, page server, verifier
- `tests/` — e2e scenarios
- `README.md` — what this component is, how to build and test it, where its contract lives
- `package.json` — e2e package manifest (Playwright)
- `playwright.config.ts` — Playwright configuration: one worker, scenarios (or the cross-browser core) then the log audit.
- `run-browsers.sh` — Runs the cross-browser core once per installed browser and writes the results table.
- `tsconfig.json` — type-checking settings of the suites
