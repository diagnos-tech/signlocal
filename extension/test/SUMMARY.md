# extension/test

- `SPEC-QUESTIONS.md` — Ambiguities of SPEC.md the tests had to assume.
- `fakes/browser.ts` — Fake `browser` namespace: runtime, tabs, i18n, events and native ports.
- `fakes/connection.ts` — Scriptable fake `Connection` for router tests.
- `fakes/protocol.ts` — Builders for app info and hello messages.
- `fakes/window.ts` — Minimal page `window` for content-script tests.
- `announce.test.ts` — Announcement posted to the page origin.
- `connection.test.ts` — Native port: hello, 3 s timeout, host missing, sharing, idle close, reconnect.
- `i18n.test.ts` — Popup message keys and placeholders per state, in all locales.
- `origin.test.ts` — `webContext`: secure contexts, loopback http, stamped origins.
- `popup-state.test.ts` — The six popup states derived from probe facts.
- `relay.test.ts` — Content relay: well-formed same-window messages only, no forged origins.
- `router-app-state.test.ts` — Router with an outdated app and a missing native host.
- `router-lifecycle.test.ts` — Router cleanup on tab close, navigation and port close.
- `router.test.ts` — Router forwarding, native ids, replies and `status`.
- `validate.test.ts` — `validatePageRequest`: accepted shapes, rejections, no smuggled fields.
- `version.test.ts` — `isOlder` vectors and malformed input.
