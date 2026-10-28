# @signlocal/sdk API reference

Sign with the visitor's certificate (smart card, USB token or a certificate installed on the computer) from
any web page, through the signlocal extension and app. Zero dependencies, typed, under 5 KB gzipped.

New here? Start with the Quickstart (top menu). Three entry points:

- **`@signlocal/sdk`**: `sign()`, `certificates()`, `status()`, `onChange()`, `installUrl()`, `fingerprint()`
  and `SignlocalError`.
- **`@signlocal/sdk/messages`**: `errorText()`, localized texts for people in 7 languages.
- **`@signlocal/sdk/testing`**: `installFakeSignlocal()`, a fake extension and app for tests and local
  development. Never ship it.

```ts
import { sign } from "@signlocal/sdk";

const { signature, certificate } = await sign({
  hash: "SHA-256",
  prepare: (certificate, { hash }) => crypto.subtle.digest(hash, signedAttributesFor(certificate)),
});
```
