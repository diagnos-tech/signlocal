# extension/src/entrypoints

- `popup/` — the toolbar popup page
- `background.ts` — Service worker (Chromium) / event page (Firefox, Safari): the only part of the extension that talks to the app.
- `content.ts` — Runs in every https page (and http on the loopback names origin.ts accepts), in all frames, at document_start: announces the extension and relays page messages to the background.
