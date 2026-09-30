# e2e

- `audit/` — checks that run after every scenario (the log audit)
- `fixtures/` — pages the e2e tests load
- `lib/` — e2e helpers: environment, keys, browser, app, page server, verifier
- `tests/` — e2e scenarios
- `README.md` — what this component is, how to build and test it, where its contract lives
- `package.json` — e2e package manifest (Playwright)
- `playwright.config.ts` — Playwright configuration: one worker, scenarios then the log audit.
- `tsconfig.json` — type-checking settings of the suites
