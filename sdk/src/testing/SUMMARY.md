# sdk/src/testing

- `index.ts` — `installFakeWebSign()` and the `FakeWebSign` controller (the public entry)
- `extension.ts` — the fake extension + app: answers status, choose and sign requests with a scripted person
- `transport.ts` — message events on the page's window, as the content script sends them
- `replies.ts` — scenarios and the frames the fake sends
- `certificate.ts` — fake certificates: X.509 v3 DER built with the test keys, wire form
- `der.ts` — the DER encodings a certificate needs
- `p256.ts` — ECDSA P-256 over a digest, RFC 6979 nonces
- `rsa.ts` — RSA PKCS#1 v1.5 and PSS over a digest (CRT)
- `bigint.ts` — BigInt helpers: bytes, modular power and inverse, Base64url
- `keys.ts` — the fixed public test keys (JWK)
- `verify.ts` — `fake.verify()` with WebCrypto
- `guard.ts` — local-origin check and console warnings that keep the fake out of production
- `types.ts` — `FakeRequest`, `CertificatePick`
