# clients/node/test

- `fixtures/` — `fake-websign.mjs`, a scripted stand-in for `websign connect`
- `api.test.ts` — smoke test of the public surface
- `certificate.test.ts` — `der`/`chain` decoded to bytes everywhere; garbled Base64 rejected
- `connect.test.ts` — connect, hello negotiation, misbehaving apps, process cleanup
- `framing.test.ts` — frame encoding and incremental decoding
- `lifetime.test.ts` — the event loop is held only while a request is open
- `locate.test.ts` — executable search order on every OS
- `methods.test.ts` — status, certificates, diagnostics, concurrent requests
- `sign.test.ts` — sign flow, stale digests, errors, AbortSignal
- `support.ts` — scenario builder and helpers for the fake app
- `version.test.ts` — library version equals package.json
