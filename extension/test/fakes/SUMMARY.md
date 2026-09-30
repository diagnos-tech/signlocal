# extension/test/fakes

- `browser.ts` — Fake `browser` namespace (a singleton across `vi.resetModules`): runtime, tabs, i18n, action, native ports.
- `connection.ts` — Scriptable fake `Connection` for router tests.
- `protocol.ts` — Builders for app info and hello messages.
- `window.ts` — Minimal page `window` for content-script tests.
