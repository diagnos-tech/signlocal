# examples/web/pades

- `sign-pdf.ts` — the flow: prepare the PDF, `sign()` with a `prepare` that builds signed attributes, embed
- `pdf.ts` — pdf-lib: signature placeholder, byte ranges, embedding the CMS, sample document
- `cms.ts` — PKI.js: PAdES signed attributes and the SignedData around the raw signature
- `main.ts` — the page: sign a sample or a chosen PDF, download the result
- `index.html` — page shell
- `pades.test.ts` — signs with the fake (EC, RSA, certificate switch) and verifies with `openssl cms -verify`
