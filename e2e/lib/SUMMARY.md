# e2e/lib

- `app.ts` — The app under test: the environment its processes get, its data and log folders, `register` and `diagnostics`.
- `firefox/` — Firefox driven over WebDriver BiDi: client, typed values, tab, launch with the temporary add-on
- `browser.ts` — Chromium with the unpacked extension in a fresh profile, the e2e app registered as its native messaging host; named installed browsers get the extension over DevTools.
- `browsers.ts` — The installed browsers a run can prove and where each keeps its default profile.
- `crash-report.ts` — The macOS crash report of an app process killed by a signal: exception, messages, crashed thread.
- `environment.ts` — What an e2e run needs (app, extension, keys, screenshots) and how each OS receives its software keys.
- `fixture.ts` — Typed calls into the fixture page, for a Playwright page or a Firefox tab.
- `installed.ts` — Registration as a person's install does it (`websign register --browser <name>`) for a named browser.
- `global-setup.ts` — Once per run: private app folders, no stale renderer marker, the SoftHSM2 token when nothing else provides keys.
- `keys.ts` — The software keys of the run, known by their certificates.
- `old-app.ts` — Puts the fake old app in place of the app's registration in a throwaway profile (AppOutdated).
- `session.ts` — The browser under test behind one interface for the cross-browser suite (Chromium family or Firefox), and the hooks that open and close it.
- `server.ts` — Serves the fixture page and the website from the repository on localhost.
- `suite.ts` — What every spec shares: environment, keys, the signature checks.
- `verify.ts` — The independent verifier (node:crypto) of every signature.
- `window.ts` — What the confirmation window shows, seen through the pictures the e2e build saves.
