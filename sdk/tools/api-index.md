# @websign/sdk API reference

Sign with the visitor's certificate (smart card, USB token or a certificate installed on the computer) from
any web page, through the SignLocal extension and app. Zero dependencies, typed, under 5 KB gzipped.

New here? Start with the Quickstart (top menu). Three entry points:

- **`@websign/sdk`**: `sign()`, `certificates()`, `status()`, `onChange()`, `installUrl()`, `fingerprint()`
  and `WebSignError`.
- **`@websign/sdk/messages`**: `errorText()`, localized texts for people in 7 languages.
- **`@websign/sdk/testing`**: `installFakeWebSign()`, a fake extension and app for tests and local
  development. Never ship it.

```ts
import { sign } from "@websign/sdk";

const { signature, certificate } = await sign({
  hash: "SHA-256",
  prepare: (certificate, { hash }) => crypto.subtle.digest(hash, signedAttributesFor(certificate)),
});
```
