# nm-e2e

End-to-end harness for the whole browser path: page, content script, service worker, native messaging, and the probe, in a real Chromium.

- `.gitignore` — ignores `node_modules/`
- `lib/` — the harness modules used by `run.mjs`
- `package-lock.json` — pinned Node dependencies
- `package.json` — scripts (`e2e`, `test`, `selftest`) and the single dependency, `playwright-core`
- `page.html` — the stand-in website: waits for the extension, exercises the host, and publishes what it saw
- `run.mjs` — entry point: starts the page server, launches Chromium with the extension, and prints the verdict
- `selftest/` — a fake probe that lets the harness itself be tested without any keystore
- `test/` — unit tests of the harness (`node --test`)
