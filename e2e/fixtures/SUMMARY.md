# e2e/fixtures

- `page.html` — the fixture page: runs the SDK as a site would and exposes `window.websignE2e` (status, certificates, sign, abort) in plain JSON
- `old-app.mjs` — a fake old app: a native messaging host answering `hello` as version 0.0.1 (AppOutdated)
