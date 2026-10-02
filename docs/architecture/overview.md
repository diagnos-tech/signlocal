# Overview

WebeSign returns a signature, made by the user's own certificate, over a hash
the caller prepared. It never sees the document, never builds a signature
format, and never talks to a chip directly: it asks the operating system (or
a PKCS#11 driver) to sign. That is why it works with any certificate the
machine can use — ICP-Brasil A1/A3, eIDAS qualified cards, Latin American
tokens — and with any format the caller builds (PAdES, CAdES, XAdES, JAdES…).

## Components

```
 website ─ @websign/sdk ─ postMessage ─▶ extension ─ native messaging ─┐
                                        (content + background)          │
 desktop program ─ @websign/desktop / websign-client ─ spawn ───────────┤
 script ─ `websign sign|choose` ─────────────────────────────────────────┤
                                                                         ▼
                              ┌───────────────────── websign (one binary) ─────────────────────┐
                              │ launch detection → host engine (session, flows, queue, consent)│
                              │        │ ports            │                       │            │
                              │   confirmation window   key store worker      device monitor │
                              │   (egui, main thread)   (KeystoreHub)         (PC/SC events) │
                              └────────────────────────────┬────────────────────────────────────┘
                                                           ▼
                             Windows CNG/CAPI · macOS Keychain/CryptoTokenKit · PKCS#11 modules
```

| Component | Folder | Language | License | Role |
|---|---|---|---|---|
| Web SDK | `sdk/` | TypeScript | Apache-2.0 | `status/certificates/sign/installUrl/onChange/fingerprint` for pages |
| Extension | `extension/` | TypeScript (WXT, MV3) | GPL-3.0-or-later | page ↔ app bridge; toolbar popup; no other UI |
| App | `app/` | Rust | GPL-3.0-or-later | the `websign` binary: host, CLI, windows |
| Core logic | `crates/websign-core` | Rust | GPL-3.0-or-later | algorithms, encodings, tolerant certificate summary, verify, dedup, display rules |
| Protocol | `crates/websign-protocol` | Rust | Apache-2.0 | wire types, framing, errors, verification code; generates TS |
| Key stores | `crates/websign-keystores` | Rust | GPL-3.0-or-later | CNG/CAPI, Keychain/CTK, PKCS#11 behind one trait |
| Devices | `crates/websign-devices` | Rust | GPL-3.0-or-later | USB/PC/SC scan, live events, `devices.json` hints |
| Host engine | `crates/websign-host` | Rust | GPL-3.0-or-later | sessions, flows, queue, consent, stores, runtime |
| Registration | `crates/websign-registration` | Rust | GPL-3.0-or-later | browser manifests, URL scheme, pre-registration |
| i18n | `crates/websign-i18n` + `i18n/` | Rust + TOML | GPL-3.0-or-later | keys, locales, plurals, dates |
| UI model | `crates/websign-ui-model` | Rust | GPL-3.0-or-later | pure view logic of both windows |
| Project IDs | `crates/websign-project` | Rust | Apache-2.0 | `project.toml` as constants |
| Node client | `clients/node` | TypeScript | Apache-2.0 | `@websign/desktop` |
| Rust client | `clients/rust` | Rust | Apache-2.0 | `websign-client` |
| Safari bridge | `safari/` | Swift | GPL-3.0-or-later | app extension inside `WebeSign.app` (direct builds, macOS 13+); starts its own sandboxed host and relays to it |
| Device hints | `devices.json` | JSON | CC0-1.0 | VID:PID/ATR → model → driver per OS |
| Packaging | `packaging/`, `scripts/install/` | shell, PowerShell, manifests | GPL-3.0-or-later | artifacts and one-line installers |
| E2E | `e2e/` | TypeScript (Playwright) | GPL-3.0-or-later | page → extension → app → software key, with screenshots |
| Automation | `xtask/` | Rust | GPL-3.0-or-later | `cargo xtask gen|check|package|screenshots` |

The crate graph is acyclic and points one way:

```
websign-project   websign-protocol
        ▲               ▲   ▲
        │         websign-core ◀── websign-ui-model
        │               ▲   ▲            ▲
 websign-devices ◀─ websign-keystores    │
        ▲               ▲                │
        └──────── websign-host ──────────┘      websign-registration   websign-i18n
                        ▲                                ▲                 ▲
                        └──────────────── app ───────────┴─────────────────┘
websign-client → websign-protocol, websign-project   (Apache-2.0 only)
```

**Why so many crates:** each has one job and a `SPEC.md`; pure crates
(`core`, `protocol`, `ui-model`, `i18n`) are blind-TDD territory and compile
without egui, wgpu or OS SDKs; platform crates hide `unsafe` behind traits
with contract tests; the Apache-2.0 crates never depend on GPL ones.

## Processes and lifetimes

Nothing runs permanently. Every process ends when its job ends.

| Started by | Arguments | Lives while | Does |
|---|---|---|---|
| a browser (native messaging) | `chrome-extension://<id>/ [--parent-window=N]` or `<manifest path> <gecko id>` | the extension keeps the port (it closes after 60 s idle) | serves pages of that browser profile |
| a desktop program | `websign connect` | stdin is open (idle exit 300 s) | serves that program |
| a script / person | `websign sign|choose|…` | one request | prints JSON, exits |
| the OS (URL) | `websign:activate` (Windows, Linux); no arguments + a `kAEGetURL` Apple Event (macOS) | registration + diagnostics window | first-run registration for store builds |
| the app menu | no arguments | the diagnostics window is open | diagnostics; silent re-registration |
| a host process | `websign diagnostics --tab …` (spawned) | the window is open | "Open diagnostics" outlives the connection |

Launch detection (`app/src/launch.rs`) is strict: browser-shaped arguments
start a host only when they name one of our extensions (the IDs of
`project.toml`, the same the manifests allow); anything else is parsed as a
command line and a mistake is a usage error (exit 2), never host mode. A
`websign:` URL names one action (`activate`); every other URL only opens
diagnostics. On macOS Launch Services starts the app without the URL and
delivers it as an Apple Event, also to a copy that already shows a window;
`app/src/platform/url_events.rs` installs the handler when AppKit posts
"will finish launching", so neither window needs code for it.

Two browsers mean two host processes, each with its own window queue; they
share state through files (consent, settings, connection records) with
locked, atomic writes. PKCS#11 modules load once per process and are never
finalized (unloading a driver with live threads crashes processes; the kit
learned this).

## Threads of a host process

| Thread | Owns | Talks through |
|---|---|---|
| main | the egui/winit event loop (macOS requires the main thread); started with the first window, kept alive hidden between requests (winit cannot create a second event loop); also opens the OS certificate viewer, modal on Windows and macOS | `UiCommand` in, `UiEvent` out; a certificate for the viewer through the window bridge |
| engine | `websign_host::Engine`: session, queue, flows — deterministic, single-threaded | one `mpsc` channel of `EngineEvent` |
| stdin reader | blocking frame reads | posts `Frame`/`Closed`/`Broken` |
| key store worker | `KeystoreHub` (key stores are not `Send`; Security.framework calls are serialized, as Chromium does); a short-lived watch per listing posts `SlowListing` after 2 s | `KeyCommand` in, `KeyReply` out |
| device monitor | `SCardGetStatusChange` loop | posts `DeviceEvent` |
| timer | 250 ms ticks while a request is open | posts `Tick` |

Signing blocks inside the key store (the OS PIN dialog can take a minute), so
it never runs on the engine thread: cancellation, disconnects and timeouts
keep working while a PIN dialog is open.

"View in system" (`docs/ux.md` §5.12) follows the same rule: the window
sends `UiEvent::ViewCertificate`, the engine takes the certificate's DER from
the last listing and hands it to `ConfirmUi::view_certificate`, and the app's
bridge passes it to the main thread, which opens the OS viewer owned by the
confirmation window. The DER never enters the window's model.

The process ends when the engine does: the client closed stdin, the
desktop idle limit passed, or the first frame was refused. A connection
that never needs a window (`status`, a remembered `choose`) never starts
the UI toolkit.

### Renderer

`app/src/ui/renderer.rs` picks the backend without re-executing, from facts
known at start: `WEBSIGN_RENDERER=wgpu|glow` when set, else glow when the
`renderer-glow` marker exists in the data folder (macOS and Linux), else
wgpu (DX12/WARP, Vulkan/llvmpipe, Metal cover RDP and GPU-less VMs; glow
does not work on Windows). The first wgpu failure for a graphics reason on
macOS or Linux writes the marker. A **host process never re-executes**: it
has already read the request from stdin, which a child could not see, so it
fails the requests on screen with `Internal` and the next launch starts
with glow. The **diagnostics window** has read nothing, so it writes the
marker and re-executes itself at once with `WEBSIGN_RENDERER=glow`.

## Data flow of one signature

1. The page calls `sign({hash, prepare})`. The SDK posts `sign.begin`.
2. The content script relays; the background validates, adds `web` from the
   browser, opens (or reuses) the port, says `hello`, forwards.
3. The host checks origin and version, queues the request, shows the window,
   and asks the key store worker for the (cached) listing.
4. The window lists usable certificates (`websign-ui-model`), preselects one.
   Remembered caller: the host sends `sign.need_digest` at once; otherwise
   after "Continue" (D11).
5. The page's `prepare(certificate)` builds the signed attributes and returns
   the digest; the SDK sends `sign.digest`; the window shows the verification
   code (the page shows the same code via `fingerprint()`).
6. After 600 ms armed, the person clicks Sign. OS keys: the OS asks for the
   PIN in a dialog owned by our window. PKCS#11 keys: our PIN field; the
   buffer is zeroized after `C_Login`; the session stays logged in (D5).
7. The worker signs, the host verifies the signature against the certificate,
   replies `sign.result`, the window shows "Signed" for 900 ms.

## State on disk

Per-user data folder (`%APPDATA%\websign`, `~/Library/Application
Support/websign`, `$XDG_CONFIG_HOME/websign`): `consent.json` (remembered
callers), `usage.json` (certificate last-use), `connections.json` (extension
pings per browser), `errors.json` (last 20 error codes), `settings.json`
(user-added drivers, onboarding). None of it contains PINs, digests or
document data; only consent and usage hold certificate fingerprints, and
they never leave the machine.

Logs live in the per-user log folder (`%LOCALAPPDATA%\websign\logs`,
`~/Library/Logs/websign`, `$XDG_STATE_HOME/websign` or
`~/.local/state/websign`), never the shared temp folder: on macOS and Linux
the folder is `0700` and `websign.log` is `0600`. At 1 MiB the file becomes
`websign.log.1`, replacing the previous one (at most 2 MiB on disk);
concurrent processes append whole lines and follow each other's rotation.
Level from `WEBSIGN_LOG` (default `info`). Every line passes a privacy
filter (`app/src/logging/redact`) on top of the rule that log calls carry
steps, codes and sizes only ([security.md](security.md#logs)).
`websign uninstall --purge` deletes the data and log folders, and only
folders whose path ends in the app's own name.

## Channels

`direct` (now, unsigned): zip/tar/deb/rpm/`.app` zip from GitHub Releases, not
sandboxed. `store` (later, D10): MSIX and Mac App Store.
Safari is not store-only: the direct `.app` already carries the Safari app extension (macOS 13+, loaded with
"Allow Unsigned Extensions" while the build is unsigned; `TODO(gustavo)`: Developer ID signing, notarization and the
App Store). Its host runs sandboxed inside the appex: Keychain works and the CryptoTokenKit query runs (no real token
tried yet); PKCS#11-only tokens may not work there.
Same code; the differences are detected at run time (package identity,
sandbox) — never build forks. See [packaging-and-release.md](packaging-and-release.md).
