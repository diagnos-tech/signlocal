# @websign/desktop

Call the SignLocal app from Node.js, Bun or Electron: the person picks a
certificate in the app's window, you supply the digest, the app signs it with
the key (PIN included). Zero runtime dependencies. Node ≥ 20.19 (ESM; `require()`
works too).

```sh
npm install @websign/desktop
```

## Quickstart

Save as `sign.mjs` and run `node sign.mjs` (the SignLocal app must be installed):

```ts
import { createHash } from "node:crypto";
import { WebSign } from "@websign/desktop";

const websign = await WebSign.connect(); // AppMissing if not installed
try {
  const { signature, certificate } = await websign.sign({
    hash: "SHA-256",
    prepare: () => createHash("sha256").update("my document").digest(),
  });
  console.log(`${certificate.displayName}: ${signature.byteLength} bytes`);
} finally {
  await websign.close();
}
```

No app yet, or in CI? Swap the first line for the fake (see below):
`const app = fakeApp(); const websign = await app.connect();`.

## Test without the app

`@websign/desktop/testing` starts a fake `websign connect` that speaks the real
protocol, so your code runs unchanged:

```ts
import { WebSignError } from "@websign/desktop";
import { fakeApp } from "@websign/desktop/testing";

const app = fakeApp({ failWith: { code: "UserCancelled" } }); // or omit to sign
const websign = await app.connect();
await websign.sign({ hash: "SHA-256", prepare: () => new Uint8Array(32) }).catch((error) => {
  console.log(error instanceof WebSignError, error.code); // true "UserCancelled"
});
await app.close();
```

Options: `certificates`, `failWith: { code, message?, when?: "choose" | "confirm" }`
(`confirm` fails after `prepare`, like a blocked PIN), `remembered`. The fake
answers `sign` with the digest itself as `signature`: it never verifies, so do
not use it to test signature validation. `app.requests()` lists what your code
sent.

## Errors

Every failure is a `WebSignError` with a stable `code` (branch on it), a
developer `message` (English), a `hint` saying what to do, and a `docsUrl`:

```ts
catch (error) {
  if (error instanceof WebSignError && error.code !== "UserCancelled") {
    console.error(`${error.code}: ${error.message}\n${error.hint}\n${error.docsUrl}`);
  }
}
```

## Good to know

- Same mental model as the web SDK (`@websign/sdk`): the same `sign({ hash,
  algorithm, certificate, prepare, signal })` options, `prepare(certificate,
  { hash, algorithm })`, the same `Certificate` shape (`notBefore`/`notAfter`
  are `Date`s) and the same error codes. `algorithm` is one name or a list in
  preference order; `certificate` preselects by `Certificate` or fingerprint.

- `prepare` throwing, an aborted `signal` or a wrong-length digest cancels the
  request in the app; you get `Aborted` (the thrown error as `cause`) or
  `InvalidRequest`. So does an app that asks for another hash or an
  algorithm outside the ones you accepted (`InvalidRequest`, before `prepare`
  runs).
- `WebSign.connect({ executable })` or `WEBSIGN_EXECUTABLE` pins the binary;
  otherwise `findExecutable()` searches `PATH` and the per-OS install
  locations.
- One connection serves one caller and one window at a time. The app exits when
  you `close()` (or when your process exits); no daemon, no port.
- `certificate.der` and `certificate.chain` are DER bytes (`Uint8Array`), ready
  for your CMS/PAdES code; `signature` is raw (RSA block, ECDSA `r || s`).
- An idle connection does not keep your program running; still, `close()` it.
- Node ≥ 20.19 (earlier 20.x cannot `require()` ESM), Bun or Electron.
- Also: `status()`, `certificates({ algorithm?, signal? })`, `openDiagnostics(tab?)`.

## Development

- API guide: [`docs/architecture/desktop-api.md`](../../docs/architecture/desktop-api.md) §7.
- Contract: [`SPEC.md`](SPEC.md). Types in `src/generated` come from
  `crates/websign-protocol` (`cargo xtask gen`).
- Scripts: `bun run typecheck`, `bun run test`, `bun run build`, `bun run smoke`
  (imports the built package as ESM and CJS).
- Example: [`examples/desktop/node-cli`](../../examples/desktop/node-cli).
- Tests drive a scripted fake `websign connect` (`test/fixtures/fake-websign.mjs`)
  for misbehaving apps, and the public fake for well-behaved ones.
- License: **Apache-2.0** ([`LICENSE`](LICENSE)).
