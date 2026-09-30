# @websign/desktop

Call the WebeSign app from Node.js, Bun or Electron: the person picks a
certificate in the app's window, you supply the digest, the app signs it with
the key (PIN included). Zero runtime dependencies.

```ts
import { createHash } from "node:crypto";
import { readFile } from "node:fs/promises";
import { WebSign } from "@websign/desktop";

const document = await readFile("contract.pdf");
const websign = await WebSign.connect(); // AppMissing if not installed
try {
  const { signature, certificate, algorithm } = await websign.sign({
    hash: "SHA-256",
    signal: AbortSignal.timeout(120_000),
    // Runs once the person picked a certificate (again if they switch). For
    // CMS/PAdES, hash the signed attributes built from `certificate.der` here.
    prepare: () => createHash("sha256").update(document).digest(),
  });
  console.log(certificate.displayName, algorithm, signature.byteLength);
} finally {
  await websign.close();
}
```

## Good to know

- Errors are `WebSignError` with a stable `code` (`UserCancelled`, `AppMissing`,
  `Timeout`, `PinLocked`, ...): branch on `code`, not on `message`.
- `prepare` throwing, an aborted `signal` or a wrong-length digest cancels the
  request in the app; you get `Aborted` (the thrown error as `cause`) or
  `InvalidRequest`.
- `WebSign.connect({ executable })` or `WEBSIGN_EXECUTABLE` pins the binary;
  otherwise `findExecutable()` searches `PATH` and the per-OS install
  locations.
- One connection serves one caller and one window at a time. The app exits when
  you `close()` (or when your process exits); no daemon, no port.
- `certificate.der` and `certificate.chain` are DER bytes (`Uint8Array`), ready
  for your CMS/PAdES code; `signature` is raw (RSA block, ECDSA `r || s`).
- An idle connection does not keep your program running; still, `close()` it.
- Node ≥ 20.19, Bun or Electron; ESM, and `require()` works too.
- Also: `status()`, `certificates(filter?)`, `openDiagnostics(tab?)`.

## Development

- API guide: [`docs/architecture/desktop-api.md`](../../docs/architecture/desktop-api.md) §7.
- Contract: [`SPEC.md`](SPEC.md). Types in `src/generated` come from
  `crates/websign-protocol` (`cargo xtask gen`).
- Scripts: `bun run typecheck`, `bun run test`, `bun run build`. Tests drive a
  fake `websign connect` (`test/fixtures/fake-websign.mjs`) from scenario files.
- License: **Apache-2.0** ([`LICENSE`](LICENSE)).
