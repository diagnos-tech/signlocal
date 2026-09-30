# e2e/lib

- `app.ts` — The app under test: the environment its processes get, its data and log folders, `register` and `diagnostics`.
- `browser.ts` — Chromium with the unpacked extension in a fresh profile, the e2e app registered as its native messaging host.
- `environment.ts` — What an e2e run needs (app, extension, keys, screenshots) and how each OS receives its software keys.
- `fixture.ts` — Typed calls into the fixture page.
- `global-setup.ts` — Once per run: private app folders, the SoftHSM2 token when nothing else provides keys.
- `keys.ts` — The software keys of the run, known by their certificates.
- `server.ts` — Serves the fixture page and the website from the repository on localhost.
- `suite.ts` — What every spec shares: environment, keys, the signature checks.
- `verify.ts` — The independent verifier (node:crypto) of every signature.
