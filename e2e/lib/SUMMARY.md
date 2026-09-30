# e2e/lib

- `app.ts` — The app under test: the environment its processes get, its data and log folders, `register` and `diagnostics`.
- `browser.ts` — Chromium with the unpacked extension in a fresh profile, the e2e app registered as its native messaging host.
- `environment.ts` — What an e2e run needs (app, extension, keys, screenshots) and how each OS receives its software keys.
- `fixture.ts` — Typed calls into the fixture page.
- `global-setup.ts` — Once per run: private app folders, the SoftHSM2 token when nothing else provides keys.
- `keys.ts` — The software keys of the run, known by their certificates.
- `old-app.ts` — Puts the fake old app in place of the app's registration in a throwaway profile (AppOutdated).
- `server.ts` — Serves the fixture page and the website from the repository on localhost.
- `suite.ts` — What every spec shares: environment, keys, the signature checks.
- `verify.ts` — The independent verifier (node:crypto) of every signature.
- `window.ts` — What the confirmation window shows, seen through the pictures the e2e build saves.
