# @websign/sdk

Sign with the visitor's certificate (smart card, USB token or a certificate
installed on the computer) from any web page, through the WebeSign extension
and app. Zero dependencies, < 5 KB.

```ts
import { sign, fingerprint } from "@websign/sdk";
const result = await sign({
  hash: "SHA-256",
  prepare: async (certificate, { algorithm }) => digestOfSignedAttributes(certificate, algorithm),
});
```

- API guide: [`docs/architecture/web-api.md`](../docs/architecture/web-api.md).
- Contract: [`SPEC.md`](SPEC.md). Wire types in `src/generated` come from
  `crates/websign-protocol`.
- Scripts: `bun run typecheck`, `bun run test`, `bun run build`.
- License: **Apache-2.0** ([`LICENSE`](LICENSE)).
