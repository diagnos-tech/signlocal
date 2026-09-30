# Desktop API

How any desktop program — a native app, an Electron app, a script — asks the
user for a signature. Same flow and same protocol as the web
([protocol.md](protocol.md)); the difference is who the caller is and how the
app is started.

## 1. Ways in

| Interface | For | Certificate-dependent digests (PAdES) |
|---|---|---|
| `websign sign` / `websign choose` | scripts, simple integrations | no: the digest is given up front |
| `websign connect` | client libraries, any language | yes: `sign.need_digest` ↔ `sign.digest` |
| `@websign/desktop` (Node, Bun, Electron) | JavaScript programs | yes (`prepare`) |
| `websign-client` (Rust crate) | Rust programs | yes (`prepare` closure) |

The app is started on demand by the caller and exits when done: no daemon,
no port.

## 2. Commands

`websign` is on `PATH` after installation (Linux packages: `/usr/bin/websign`;
Windows: the MSIX alias or `%LOCALAPPDATA%\Programs\WebeSign\websign.exe`;
macOS: `/Applications/WebeSign.app/Contents/MacOS/websign` — see
[packaging-and-release.md §Install locations](packaging-and-release.md#install-locations)).

| Command | Purpose | Output |
|---|---|---|
| `websign` (no arguments) | open diagnostics (what the app menu entry runs) | window |
| `websign install [--browser B]… [--system] [--dry-run] [--json]` | register with browsers, the `websign:` scheme, extension pre-registration (Windows), app-menu entry (Linux) | text or JSON report |
| `websign uninstall [--purge] [--system] [--dry-run] [--json]` | undo `install` (`--purge` also deletes settings and remembered sites) | text or JSON |
| `websign register [--browser B]… [--uninstall] [--scope user\|system] [--user-data-dir DIR] [--manifest-dir DIR] [--dry-run] [--json]` | manifests only (tests, support) | one line per target |
| `websign doctor [--json]` | the "Copy diagnostics" report | text (docs/ux.md §8.7) or JSON |
| `websign diagnostics [--tab browsers\|devices\|certificates\|help]` | open the diagnostics window | window |
| `websign sign --hash H (--digest D \| --digest-file PATH\|-) [--algorithm A]… [--certificate FP]` | one signature | JSON (§3) |
| `websign choose [--algorithm A]…` | let the person pick a certificate | JSON (§3) |
| `websign connect` | framed protocol on stdin/stdout | frames |
| `websign version [--json]` | version | `websign 1.4.0 (protocol 1)` or `AppInfo` JSON |

Values: `--hash SHA-256|SHA-384|SHA-512`; `--algorithm ECDSA|RSASSA-PKCS1-v1_5|RSASSA-PSS`
(repeatable, preference order); `--digest` accepts hex (any case, `:`
allowed) or padded standard Base64; `--certificate` a SHA-256 fingerprint in
hex.

Browsers `B`: `all`, `chrome`, `chromium`, `edge`, `brave`, `vivaldi`, `opera`,
`firefox`.

The app also repeats `register` silently on every start of the diagnostics
window, so a browser installed later is picked up without reinstalling.

Manifests allow only the extension IDs in `project.toml`; there is no flag to
add others, because the app refuses to serve any other ID (T2 in
[security.md](security.md)).

## 3. JSON output of `sign` and `choose`

stdout carries **exactly one JSON object**, the final protocol message without
envelope; human text goes to stderr.

```json
{"type":"sign.result","hash":"SHA-256","algorithm":"ECDSA","signature":"…","certificate":{…}}
{"type":"choose.result","certificates":[{…}]}
{"type":"error","code":"UserCancelled","message":"the person cancelled"}
```

## 4. Exit codes

| Code | Meaning |
|---|---|
| 0 | success |
| 1 | `Internal` |
| 2 | usage error (bad flags; clap) |
| 3 | `UserCancelled`, `Aborted` |
| 4 | `Timeout` |
| 5 | `NoCertificates` |
| 6 | `CertificateUnavailable` |
| 7 | `CertificateNotValid` |
| 8 | `UnsupportedAlgorithm` |
| 9 | `PinLocked` |
| 10 | `PinIncorrect` (unattended tests only) |
| 11 | `TokenRemoved` |
| 12 | `DriverFailure` |
| 13 | `Busy` |
| 14 | `InvalidRequest`, `InsecureOrigin` |
| 15 | `AppOutdated`, `ClientOutdated`, `AppMissing`, `ExtensionMissing`, `ExtensionOutdated` |

Defined once in `websign_protocol::ErrorCode::exit_code`.

## 5. `websign connect`

1. The program spawns `websign connect` with stdin/stdout as pipes (stderr may
   be inherited or discarded).
2. Before reading, the app identifies the **parent process** (§6).
3. The program sends `hello` (no `browser` field), then any requests without
   `web` ([protocol.md §4](protocol.md#4-message-catalog)).
4. Closing stdin ends the session; the app also exits after 300 s without a
   request. One process serves one caller; start several for parallel work
   (each has its own window queue).

A minimal client in any language: write `u32` little-endian length + JSON,
read the same.

## 6. Caller identity and consent

The window shows the calling program instead of a web origin:

- **name**: product name from the executable's metadata (Windows version
  resource `FileDescription`/`ProductName`; macOS `CFBundleName` of the
  enclosing `.app`; Linux the `Name` of a `.desktop` file whose `Exec` matches),
  else the file name;
- **detail**: "Signed by {signer}" when the OS verified a code signature
  (Windows Authenticode leaf CN, or "Microsoft Windows" for programs signed
  through a system catalog such as `cmd.exe`; macOS team ID + identifier,
  checked on the parent's audit token when it is our stdin peer), else the
  executable path with an "Unverified program" warning;
- a script host (interpreters, shells and terminal hosts: `node`, `python`,
  `bash`, `zsh`, `cmd.exe`, `powershell`, `Terminal`, …) is shown as "a script
  run by {program}", because every script it runs shares its identity; the
  list lives in `websign-core` `present/caller/script_hosts.rs`.

"Remember this program" stores consent under `app:<signer>` for signed
programs (survives updates that move the binary) or `path:<executable>`
otherwise. Script hosts can never be remembered (no checkbox; a stored record
is ignored). Consent is per program and certificate fingerprint: a remembered
program gets `choose` without a window for the certificates in its record, and
any other certificate needs **Continue**; every signature still asks for
confirmation. Revocation: Diagnostics › Browsers › Allowed
programs.

## 7. Client libraries

Both libraries follow the web SDK's model ([web-api.md](web-api.md)): the
same options (`hash`, `algorithm` as one name or a preference-ordered set,
`certificate` to preselect by certificate or fingerprint), `prepare`
receiving the certificate plus `{ hash, algorithm }`, and the same error
codes. Each checks every `sign.need_digest` before `prepare` runs: a hash
other than the requested one, or an algorithm outside the requested set,
cancels the request with `InvalidRequest`.

### `@websign/desktop` (Apache-2.0, zero runtime deps)

```ts
import { WebSign } from "@websign/desktop";

const websign = await WebSign.connect();            // AppMissing if not installed
const result = await websign.sign({
  hash: "SHA-256",
  algorithm: ["ECDSA", "RSASSA-PSS"],               // optional; one name or a list
  prepare: async (certificate, { algorithm }) => digestFor(certificate.der, algorithm),
});
await websign.close();
```

Also `status()`, `certificates({ algorithm?, signal? })`,
`openDiagnostics(tab?)`, `findExecutable()`. The public types are the SDK's
(`Certificate`, `SignOptions`, `SignResult`, `PrepareContext`,
`HashAlgorithm`, `SignatureAlgorithm`): `Certificate.der`/`chain` and
`SignResult.signature` are `Uint8Array`, `notBefore`/`notAfter` are `Date`.
ESM, `require()` too, Node ≥ 20.19. Contract:
[`clients/node/SPEC.md`](../../clients/node/SPEC.md).

### `websign-client` (Rust, Apache-2.0)

```rust
use websign_client::{Client, HashName, SignOptions, SignatureAlgorithmName};

let mut client = Client::connect()?;                 // ClientError::AppMissing if not installed
let options = SignOptions::new(HashName::Sha256).algorithm(SignatureAlgorithmName::Ecdsa);
let result = client.sign(options, |certificate, context| {
    Ok(digest_for(certificate.der.as_bytes(), context.algorithm)) // Err(String) cancels in the app
})?;
let signature: &[u8] = result.signature.as_bytes();
// Dropping `client` closes the app's stdin and reaps it.
```

Also `status()`, `certificates(&[algorithm])`, `open_diagnostics(tab)`,
`find_executable()`, `connect_with(ConnectOptions)`; `SignOptions` builders
`algorithms([..])`, `certificate(&chosen)`, `fingerprint(fp)`. Types are the
protocol crate's, re-exported (with `ErrorCode`): Base64 fields are decoded
on arrival (`Certificate.der`/`chain` and `SignResult.signature` are
`Base64Bytes`, `as_bytes() -> &[u8]`), `not_before`/`not_after` are Unix
seconds. `Client` is `Send`; `ClientError` is `Send + Sync` with `code()`
returning the protocol's `ErrorCode`. Blocking API over `std::process`;
depends on the Apache-2.0 crates `websign-protocol` (the wire contract) and
`websign-project` (product and executable names), both self-contained and
publishable. Contract: [`clients/rust/SPEC.md`](../../clients/rust/SPEC.md).

## 8. The `websign:` URL scheme

`websign:activate` (from the website's `/activate` page) registers the app
with every browser and opens diagnostics with "Getting started". It exists
because store packages run no install scripts. Any other `websign:` URL
silently repairs the registrations that exist and opens diagnostics; URLs
never carry data the app acts on. Both paths do the same work whether the URL
arrives as an argument or as an event.

Windows and Linux pass the URL as the last argument. macOS delivers it as an
Apple Event (`kAEGetURL`), which the app handles once it has finished
launching; `-psn_` arguments are ignored.
