# @websign/desktop — specification

Surface: [`docs/architecture/desktop-api.md`](../../docs/architecture/desktop-api.md)
§7. Tests use a fake `websign connect` (a Node script speaking the protocol
from a scenario file).

## 1. `findExecutable()`

`WEBSIGN_EXECUTABLE` if it exists; else `websign`/`websign.exe` on `PATH`;
else, in order: Windows `%LOCALAPPDATA%\Programs\SignLocal\websign.exe`,
`%LOCALAPPDATA%\Microsoft\WindowsApps\websign.exe`; macOS
`/Applications/SignLocal.app/Contents/MacOS/websign`,
`~/Applications/SignLocal.app/Contents/MacOS/websign`, `~/.local/bin/websign`
(the symlink the direct macOS install may create); Linux `/usr/bin/websign`,
`~/.local/bin/websign`. `undefined` when none exists. "Exists" means a
regular file with the execute bit; on Windows any non-directory, because the
`WindowsApps` MSIX alias is a reparse point that `stat` reports as a link.

## 2. Connection and framing

- `connect()`: spawn `[exe, "connect"]` with `stdio: ["pipe","pipe","ignore"]`;
  spawn error or immediate exit → `AppMissing`. Send `hello` with
  `client {name:"@websign/desktop", version}` and `protocols {min:1,max:1}`;
  wait ≤ 10 s for the `hello` reply. An `error` answering `hello` must carry the
  `v` that `hello` carried and rejects with its code (`ClientOutdated`,
  `AppOutdated`, `InvalidRequest`); an `error` at any other `v`, or any other
  non-`hello` answer, is `Internal` and the child is killed.
- Frames: `u32` little-endian + UTF-8 JSON; `FrameDecoder` handles frames
  split across chunks and several frames per chunk; > 1 MiB → connection
  error.
- Ids: `"n" + counter`.
- Child exit with requests open → reject them with `Internal`
  ("the app exited").
- `close()`: end stdin, wait for exit (≤ 5 s, then kill; on Windows every
  kill is `TerminateProcess`). Idempotent; rejects requests in flight with
  `Internal`.
- Event loop: the child and its pipes keep Node alive only while a request is
  in flight or `close()` is waiting; an idle, unclosed connection lets the
  program exit, and a process `exit` handler kills the child (the handler is
  removed when the child exits).

## 3. Methods

Same options, flows and error codes as the SDK (§5–§6 of `sdk/SPEC.md`)
without `web` and without discovery: `status`, `certificates({ algorithm?,
signal? })`, `sign({ hash, algorithm?, certificate?, prepare, signal? })`
(with stale-seq handling, `cancel` on throw or abort),
`openDiagnostics(tab)` → `done`.

- Options are checked before anything is sent, each failure `InvalidRequest`:
  `hash` is SHA-256/384/512; `algorithm` is one known name or a non-empty
  list of them (deduplicated, order kept, sent as `algorithms`); `certificate`
  is a `Certificate` or its fingerprint (64 lowercase hex digits; only the
  fingerprint is sent); `prepare` is a function.
- `certificates()`: an empty `choose.result` → `NoCertificates`.
- Each `need_digest` is checked before `prepare` runs: its `hash` must be the
  requested one and its `algorithm` must be in the requested set (any known
  algorithm when none was given); otherwise `cancel` and `InvalidRequest`.
- `prepare(certificate, { hash, algorithm })` must return exactly 32/48/64
  bytes for SHA-256/384/512 (`Uint8Array`, any typed-array view, or
  `ArrayBuffer`); otherwise `cancel` and `InvalidRequest`. A throw → `cancel`
  and `Aborted` with the error as `cause`. Digests and failures for a `seq`
  older than the latest are dropped.
- `signal` already aborted → `Aborted`, nothing sent; aborted later →
  `cancel` sent, `Aborted` at once; the listener is removed when the call
  settles.

## 4. Public types

Shaped like the SDK's (`sdk/src/types.ts`): `HashAlgorithm`,
`SignatureAlgorithm`, `PrepareContext`, `SignOptions`, `CertificateOptions`,
`SignResult`, and a `Certificate` whose `der` is `Uint8Array`, `chain`
`readonly Uint8Array[]` and `notBefore`/`notAfter` `Date`, decoded once when
the message arrives (`certificates()`, `prepare`, `SignResult.certificate`);
the other fields as sent. `SignResult.signature` is `Uint8Array`. Base64
that is not canonical padded RFC 4648 §4, a missing `chain` or a non-numeric
validity date fails the request with `Internal` (and `cancel` inside `sign`).

## 5. Packaging

ESM only, zero runtime dependencies, Node ≥ 20.19 (Bun, Electron with such a
Node). The `exports` `default` condition lets CommonJS `require()` load it
(Node's `require(esm)`); `types` points at the declarations.
