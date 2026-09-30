# extension/test

- `announce.test.ts` — Announcement posted to the page origin, synchronously.
- `browser-name.test.ts` — Browser names from UA-CH brands and user agents.
- `connection.test.ts` — Native port: hello, reasons, 3 s timeout, host missing, sharing, idle close with open requests, reconnect.
- `i18n.test.ts` — Popup message keys and placeholders per state, in all locales.
- `manifest.test.ts` — Manifest per target and channel (permissions, pinned ID, CSP, Gecko) and content-script matches.
- `origin.test.ts` — `webContext`: secure contexts, loopback http, stamped origins.
- `popup-handler.test.ts` — Background answers to the popup, sender checks and the "!" badge.
- `popup-state.test.ts` — The six popup states derived from probe facts.
- `popup-view.test.ts` — Popup DOM per state (happy-dom): texts, links, actions, focus and accessibility.
- `relay-handler.test.ts` — Background relay side: sender checks, origin from the browser, reply routing, document goodbyes.
- `relay.test.ts` — Content relay: well-formed same-window messages only, no forged fields, replies bound to the document.
- `router-app-state.test.ts` — Router with an outdated app and a missing native host.
- `router-lifecycle.test.ts` — Router cleanup on tab close, navigation, document gone and port close.
- `router.test.ts` — Router forwarding, native ids, tab isolation on the shared port, replies and `status`.
- `validate.test.ts` — `validatePageRequest`: accepted shapes, rejections, unknown fields refused.
- `version.test.ts` — `isOlder` vectors and malformed input.
- `fakes/` — In-memory stand-ins for the WebExtension APIs, the native connection, protocol messages and the page `window`.
