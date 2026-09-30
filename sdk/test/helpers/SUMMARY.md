# sdk/test/helpers

- `env.ts` — loads a fresh SDK against the fake window; fake-timer hooks; outcome capture
- `expect.ts` — assertions for WebSignError outcomes
- `fake-script.ts` — fake content script: announces, records requests, replies
- `fake-window.ts` — fake window recording and looping back postMessage
- `fixtures.ts` — wire certificates, replies, error codes
- `sign.ts` — begin a sign() and read sent digests
