# clients/node/test

- `errors.test.ts` — every code has a hint and the docs link; message, name and details unchanged
- `fixtures/` — `fake-websign.mjs`, a scripted stand-in for `websign connect`
- `api.test.ts` — smoke test of the public surface
- `certificate.test.ts` — `der`/`chain` decoded to bytes and validity to `Date` everywhere; garbled input rejected
- `connect.test.ts` — connect, hello negotiation, misbehaving apps, process cleanup
- `framing.test.ts` — frame encoding and incremental decoding
- `lifetime.test.ts` — the event loop is held only while a request is open
- `locate.test.ts` — executable search order on every OS
- `methods.test.ts` — status, certificates, diagnostics, concurrent requests
- `sign.test.ts` — sign flow, stale digests, errors, AbortSignal
- `support.ts` — scenario builder and helpers for the fake app
- `testing.test.ts` — the public fake app: sign, filters, failures, length check, cleanup
- `version.test.ts` — library version equals package.json
