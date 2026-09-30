# @websign/sdk web examples

Four ways to add WebeSign signing to a web page. Each runs **without the extension**: it installs the SDK's
testing fake (`@websign/sdk/testing`), which signs with public test keys and has a panel in the corner to
switch the visitor's setup (app missing, outdated…) and script the next outcome (the person cancels, the PIN
is locked…). Add `?real` to the address to use the real WebeSign extension and app instead.

| Folder | Shows |
|---|---|
| [`vanilla/`](vanilla/index.html) | One HTML file, no framework: status, sign, errors |
| [`react/`](react/) | A copy-paste `useWebSign()` hook: live status, `sign` with progress, error, cancel on unmount |
| [`vue/`](vue/) | The same as a Vue composable |
| [`pades/`](pades/) | Signing a PDF (PAdES B-B): pdf-lib reserves the signature, PKI.js builds the signed attributes and the CMS, WebeSign signs their hash, the raw signature goes back into the PDF |

## Run

```sh
cd examples/web
bun install
bun run dev        # http://localhost:5173/ lists the examples
bun run check      # typecheck, the PAdES test (verified with OpenSSL), production build
```

The examples import `@websign/sdk` from this repository's `sdk/src` (aliases in `vite.config.ts` and
`tsconfig.json`). In your project, `bun add @websign/sdk` (or npm, pnpm, yarn) and import it as shown.

## Where the PDF library meets WebeSign

[`pades/sign-pdf.ts`](pades/sign-pdf.ts) is the whole flow in 40 lines:

1. `prepare()` in [`pdf.ts`](pades/pdf.ts) adds an empty signature field with room for the CMS, and
   `signedContent()` returns every byte except that room; its SHA-256 is the `messageDigest`.
2. `sign({ hash: "SHA-256", prepare })`: once the person picks a certificate, your `prepare` builds the
   signed attributes for it ([`cms.ts`](pades/cms.ts): contentType, messageDigest, signing-certificate-v2)
   and returns the SHA-256 of their DER. It may run again if the person switches certificate, so keep one
   set of attributes per certificate fingerprint.
3. `result.signature` is raw (ECDSA r‖s, like WebCrypto); PKI.js turns it into the DER CMS wants, the
   SignedData goes into the reserved room, and the PDF is done.

[`pades/pades.test.ts`](pades/pades.test.ts) signs with the fake and verifies the result with
`openssl cms -verify`.

## Never ship the fake

`@websign/sdk/testing` is loaded with a dynamic `import()` behind a check, so production bundles of the
real flow never contain it. It also refuses non-local origins and warns on the console when installed.

License: Apache-2.0, like the SDK. pdf-lib (MIT) and PKI.js (BSD-3-Clause) are dependencies of these
examples only, never of `@websign/sdk`.
