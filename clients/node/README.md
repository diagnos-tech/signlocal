# @websign/desktop

Call the WebeSign app from Node.js, Bun or Electron. Zero runtime
dependencies.

```ts
import { WebSign } from "@websign/desktop";
const websign = await WebSign.connect();
const { signature, certificate } = await websign.sign({
  hash: "SHA-256",
  prepare: async (cert, algorithm) => digestFor(cert, algorithm),
});
await websign.close();
```

- API guide: [`docs/architecture/desktop-api.md`](../../docs/architecture/desktop-api.md) §7.
- Contract: [`SPEC.md`](SPEC.md). Types in `src/generated` come from
  `crates/websign-protocol` (`cargo xtask gen`).
- Scripts: `bun run typecheck`, `bun run test`, `bun run build`.
- License: **Apache-2.0** ([`LICENSE`](LICENSE)).
