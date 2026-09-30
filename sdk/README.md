# @websign/sdk

Sign with the visitor's certificate (smart card, USB token or a certificate
installed on the computer) from any web page, through the WebeSign extension
and app. Zero dependencies, ESM, typed, under 5 KB gzipped.

```ts
import { sign, WebSignError } from "@websign/sdk";

// Your PAdES/CAdES code: build the signed attributes for this certificate
// (they contain it, e.g. signing-certificate-v2) and return their DER.
declare function buildSignedAttributes(certificateDer: Uint8Array, algorithm: string): Uint8Array;

try {
  const result = await sign({
    hash: "SHA-256",
    prepare: async (certificate, { hash, algorithm }) => {
      const signedAttributes = buildSignedAttributes(certificate.der, algorithm);
      return crypto.subtle.digest(hash, signedAttributes); // exactly 32 bytes for SHA-256
    },
  });
  // Embed in your CMS: result.signature (RSA block, or ECDSA r‖s),
  // result.certificate.der, result.certificate.chain, result.algorithm.
} catch (error) {
  if (error instanceof WebSignError) console.warn(error.code); // e.g. "UserCancelled"
}
```

`prepare` runs after the person picks a certificate in the app's window (PAdES
and CAdES put the certificate inside the signed data). It may run again if
they switch certificate; only the last digest is signed. Return exactly 32, 48
or 64 bytes (`Uint8Array` or `ArrayBuffer`) for SHA-256, SHA-384 or SHA-512.

## Is it installed?

```ts
import { installUrl, onChange, status } from "@websign/sdk";

const { ready } = await status(); // never rejects, never opens a window
if (!ready) link.href = installUrl(); // the store of this browser
const stop = onChange((s) => (button.disabled = !s.ready)); // stop() to unsubscribe
```

## Errors

Every rejection is a `WebSignError` with a stable `code`:

```ts
import { sign, WebSignError } from "@websign/sdk";
import { errorText } from "@websign/sdk/messages"; // optional, localized

try {
  await sign({ hash: "SHA-256", prepare });
} catch (error) {
  if (error instanceof WebSignError && error.code !== "UserCancelled") {
    show(errorText(error.code, navigator.language) ?? error.message);
  }
}
```

Cancel with an `AbortSignal` (`sign({ ..., signal })`); the window closes and
the promise rejects with `Aborted`. If `prepare` throws, the request is
cancelled and rejects with `Aborted` and your error as `cause`.

## Other calls

- `certificates()` — the certificate the person chooses (never the computer's list).
- `fingerprint(digest)` — the verification code the app shows, to display next to your button.

## Notes

- Only messages from the page's own window and origin, from the extension's
  content script and with the expected shape are accepted; the SDK posts only
  to its own origin, never `"*"`, and never logs.
- The SDK speaks page protocol 1 and rejects an extension that does not
  (`ClientOutdated`: update `@websign/sdk`; `ExtensionOutdated`: the person
  updates the extension).
- Safe to import during server-side rendering: nothing touches `window` until
  a call, and without one `status()` says nothing is installed.
- Batch signing is not offered: each signature is one confirmation.

## Development

- API guide: [`docs/architecture/web-api.md`](../docs/architecture/web-api.md).
- Contract: [`SPEC.md`](SPEC.md). Wire types in `src/generated`, the error
  texts in `src/messages.gen.ts` and `src/project.ts` come from `cargo xtask gen`.
- Scripts: `bun run typecheck`, `bun run test`, `bun run build`, then
  `bun run size` (fails when a consumer bundle of `dist/` reaches 5 KB gzip).
- License: **Apache-2.0** ([`LICENSE`](LICENSE)).
