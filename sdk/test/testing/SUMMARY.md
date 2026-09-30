# sdk/test/testing

- `dom.ts` — a browser-like window (message events with source and origin) and a fresh SDK + fake per test
- `flow.test.ts` — signatures for every algorithm/hash verified by Node's crypto, X.509 validity, determinism
- `scenarios.test.ts` — every scenario, failNext, no/expired/unknown certificates, switch, abort, latency
- `guard.test.ts` — origin guard, console warnings, no window, not reachable from the main entry
- `node-crypto.d.ts` — the slice of node:crypto the tests use (no @types/node)
