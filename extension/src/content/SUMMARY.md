# extension/src/content

- `announce.ts` — Announces the extension to the page at start, on load and on `discover`.
- `relay.ts` — Page to background relay: accepts only same-window, same-origin messages with source "websign-page", rebuilds requests with `shared/validate.ts`, and posts replies back with source "websign-extension" to the page's own origin.
