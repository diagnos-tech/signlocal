# clients/node/src/testing

- `app-process.js` — the fake app itself: a plain-JavaScript child speaking the framed protocol
- `fake-app.ts` — `fakeApp()`: writes the fake's config, exposes `connect()`, `requests()` and `close()`
- `index.ts` — the `@websign/desktop/testing` entry point
- `sample-certificate.ts` — the fictional certificate the fake offers by default
