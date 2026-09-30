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
| `websign register [--browser B]… [--uninstall] [--scope user\|system] [--user-data-dir DIR] [--extension-id ID]… [--manifest-dir DIR] [--dry-run] [--json]` | manifests only (tests, support) | one line per target |
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
- a shell parent (`bash`, `zsh`, `cmd.exe`, `powershell`, `Terminal`) is shown
  as itself — the person typed the command.

"Remember this program" stores consent under `app:<signer>` for signed
programs (survives updates that move the binary) or `path:<executable>`
otherwise. Remembered programs get `choose` without a window; every signature
still asks for confirmation. Revocation: Diagnostics › Browsers › Allowed
programs.

## 7. Client libraries

### `@websign/desktop` (Apache-2.0, zero runtime deps)

```ts
import { WebSign } from "@websign/desktop";

const websign = await WebSign.connect();            // AppMissing if not installed
const result = await websign.sign({
  hash: "SHA-256",
  prepare: async (certificate, algorithm) => digestFor(certificate.der, algorithm),
});
await websign.close();
```

Also `status()`, `certificates(filter?)`, `openDiagnostics(tab?)`,
`findExecutable()`. Types are the generated protocol types, except that the
Base64 fields are decoded once on arrival: `Certificate.der`/`chain` and
`SignResult.signature` are `Uint8Array`. ESM, `require()` too, Node ≥ 20.19. Contract: [`clients/node/SPEC.md`](../../clients/node/SPEC.md).

### `websign-client` (Rust, Apache-2.0)

```rust
use websign_client::{Client, HashName, SignOptions};

let mut client = Client::connect()?;                 // ClientError::AppMissing if not installed
let result = client.sign(SignOptions::new(HashName::Sha256), |certificate, algorithm| {
    Ok(digest_for(certificate.der.as_bytes(), algorithm)) // Err(String) cancels in the app
})?;
let signature: &[u8] = result.signature.as_bytes();
// Dropping `client` closes the app's stdin and reaps it.
```

Also `status()`, `certificates(filter)`, `open_diagnostics(tab)`,
`find_executable()`, `connect_with(ConnectOptions)`. Types are the protocol
crate's; Base64 fields are decoded on arrival: `Certificate.der`/`chain` and
`SignResult.signature` are `Base64Bytes` (`as_bytes() -> &[u8]`,
`into_bytes() -> Vec<u8>`). `Client` is `Send`; `ClientError` is
`Send + Sync` with `code()` returning the protocol's `ErrorCode`. Blocking
API over `std::process`; depends on the Apache-2.0 crates `websign-protocol`
(the wire contract) and `websign-project` (product name and executable
name). Contract: [`clients/rust/SPEC.md`](../../clients/rust/SPEC.md).

## 8. The `websign:` URL scheme

`websign:activate` (from the website's `/activate` page) registers the app
with every browser and opens diagnostics with "Getting started". It exists
because store packages run no install scripts. Any other `websign:` URL only
opens diagnostics; URLs never carry data the app acts on.

Windows and Linux pass the URL as the last argument. macOS delivers it as an
Apple Event (`kAEGetURL`), which the app handles once it has finished
launching; `-psn_` arguments are ignored.
