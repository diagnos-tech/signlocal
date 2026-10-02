# extension/test/fakes

- `appex.ts` — In-memory Safari appex relay (sessions, parked polls, host that speaks or exits) behind the fake `sendNativeMessage`.
- `browser.ts` — Fake `browser` namespace (a singleton across `vi.resetModules`): runtime, tabs, i18n, action, native ports, `sendNativeMessage` and the extension URL scheme.
- `connection.ts` — Scriptable fake `Connection` for router tests.
- `protocol.ts` — Builders for app info and hello messages.
- `window.ts` — Minimal page `window` for content-script tests.
