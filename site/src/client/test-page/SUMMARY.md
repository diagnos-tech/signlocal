# site/src/client/test-page

The /test/ page's logic, ported from the former site and typed.

- `crypto.ts` — public key from a certificate and signature check with WebCrypto (pure)
- `main.ts` — detects the setup, signs the sample text, verifies it, copies the summary
- `render.ts` — draws the setup state, the result and the error
- `state.ts` — the sample text, the setup states and the support summary (pure)
