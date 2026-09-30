# e2e/tests

- `cancel.spec.ts` — Scenario 5: Cancel in the window; the page aborting, closing or navigating shows "site cancelled".
- `consent.spec.ts` — Scenarios 2 and 3: choose mode on a new site, D11 (cancel before Continue), a remembered site without a window.
- `desktop.spec.ts` — Scenario 7: `@websign/desktop` through `websign connect`, and `websign sign`.
- `detection.spec.ts` — Scenario 6: ready, ExtensionMissing, AppMissing, AppOutdated (fake old app).
- `sign.spec.ts` — Scenario 1: every hash with RSA PKCS#1 v1.5, RSA-PSS and ECDSA P-256/384/521, the wrong digest length, the website's test page.
- `window.spec.ts` — The diagnostics window per tab, saved by the app's e2e hook.
