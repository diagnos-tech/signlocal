# Protocol

The one message catalog every party speaks: the browser extension, desktop
client libraries, `websign connect`, and the Safari relay. Defined once in Rust
([`crates/websign-protocol`](../../crates/websign-protocol/)); the TypeScript
types of the SDK, the extension and the Node client are generated from it
(`cargo xtask gen`), and CI fails when they are stale. The crate's
[`SPEC.md`](../../crates/websign-protocol/SPEC.md) holds the parsing rules and
test vectors; this page explains the design.

## Contents

1. [Transports and framing](#1-transports-and-framing)
2. [Envelope](#2-envelope)
3. [Versioning and negotiation](#3-versioning-and-negotiation)
4. [Message catalog](#4-message-catalog)
5. [The sign flow](#5-the-sign-flow)
6. [Caller identity](#6-caller-identity)
7. [Errors](#7-errors)
8. [Limits and timeouts](#8-limits-and-timeouts)
9. [Page messages](#9-page-messages)
10. [Evolution rules](#10-evolution-rules)

## 1. Transports and framing

| Transport | Who connects | Who starts the app | Framing | Caller identity |
|---|---|---|---|---|
| Native messaging | our extension (Chrome, Edge, Brave, Opera, Vivaldi, Firefox) | the browser, with the extension origin as argument | 4-byte length + UTF-8 JSON | page origin from the browser (`web` field), extension ID from the launch arguments |
| `websign connect` | a desktop program (directly or through `@websign/desktop` / `websign-client`) | that program, as a child process | same | the parent process, identified by the OS |
| Safari relay (store channel, later) | the Safari appex | the appex (`NSWorkspace`) when nothing listens | same, over a Unix socket in the app group container | page origin from Safari (`sender.url`) |
| Page messages | a web page using `@websign/sdk` | — (the extension relays) | `window.postMessage` objects | none: the extension attaches it |

**Framing.** Every message is a `u32` byte length followed by that many bytes
of UTF-8 JSON (one object). Native messaging defines the length in native byte
order; every supported target is little-endian, so desktop clients always
write little-endian. Promoted from the kit's proven `framing.rs`.

**Why one catalog for all transports:** the host engine has one dispatcher;
a client library is a thin framing layer over the same types; a fix in the
flow fixes every caller.

**Why no local server:** nothing listens, no port is open, no daemon runs
(brief decision 4). The app lives only while a connection is open.

## 2. Envelope

Every message is a flat JSON object:

```json
{ "v": 1, "id": "7", "type": "sign.begin", "hash": "SHA-256" }
```

| Field | Rule |
|---|---|
| `v` | Protocol version of this message's schema (§3). Integer ≥ 1. |
| `id` | 1–64 characters from `[A-Za-z0-9._:-]`. Chosen by the client, unique among the requests it has open on this connection. Replies and continuations carry the id of the request they belong to. |
| `type` | One of the catalog's types (§4). Unknown → `InvalidRequest`. |
| everything else | The type's fields, camelCase. **Unknown fields are refused** (`InvalidRequest`). Optional fields are omitted, never `null`. |

Parsing is strict (`websign_protocol::parse_client_message`): a hostile page
must not be able to smuggle data through fields the app ignores today and
reads tomorrow. Every message has exactly one spelling: a repeated object key,
`null` for an absent field, an array where an object belongs, or `1.0` for an
integer is refused, so the extension and the app can never read the same bytes
differently (`crates/websign-protocol/SPEC.md` §1). When a frame is malformed
but has a readable `id`, the error reply goes to that id; without one, the app
logs and closes the connection.

## 3. Versioning and negotiation

- A protocol version is **one integer**. Any change to the catalog — even an
  added optional field — is a new version, because parsing is strict.
- Each party supports a contiguous range `{min, max}`. The app keeps old
  versions parseable for at least 12 months after the next one ships.
- `hello` is the first message of every connection. Any other first message →
  `InvalidRequest`, then the connection closes. A second `hello` →
  `InvalidRequest`; the connection stays open. The client's `hello` has `v`
  within its own `protocols`; the app's reply has `v` equal to `protocol`.
- An `error` answering the first frame (a refused `hello`, or a first message
  that is not `hello`) carries **that frame's `v`**
  (`websign_protocol::refusal_version`), so the caller can always read it,
  even when the two ranges do not overlap. The `error` message's shape and the
  codes that can refuse `hello` (`InvalidRequest`, `ClientOutdated`,
  `AppOutdated`, `Internal`) never change between versions. Callers parse the
  answer to `hello` with `websign_protocol::parse_hello_reply`.
- Negotiation (`websign_protocol::negotiate`): the highest version in both
  ranges. `client.max < app.min` → `ClientOutdated` (reported to pages as
  `ExtensionOutdated`); `app.max < client.min` → `AppOutdated`.
- Independently, the extension compares the app's semantic version with its
  `MIN_APP_VERSION` (from `project.toml`) and reports `AppOutdated` when older.
  This catches app bugs fixed without a protocol change.

```json
→ {"v":1,"id":"h","type":"hello","client":{"name":"websign-extension","version":"1.4.2"},
   "protocols":{"min":1,"max":1},
   "browser":{"name":"chrome","version":"129.0","reason":"page"}}
← {"v":1,"id":"h","type":"hello","protocol":1,
   "app":{"version":"1.4.0","protocols":{"min":1,"max":1},"os":"windows","arch":"x86_64","channel":"direct"}}

→ {"v":1,"id":"h","type":"hello","client":{"name":"websign-client","version":"0.1.0"},"protocols":{"min":1,"max":1}}
← {"v":1,"id":"h","type":"error","code":"ClientOutdated","message":"the app needs protocol 2 or newer",
   "details":{"installed":"1","required":"2"}}
```

## 4. Message catalog

Direction: → client to app, ← app to client. *Request* = starts something and
gets exactly one final reply. *Continuation* = refers to an open request, no
reply of its own. *Event* = non-final message from the app.

| Type | Dir | Kind | Fields | Final reply |
|---|---|---|---|---|
| `hello` | → | request | `client {name, version}`, `protocols {min,max}`, `browser?` (native messaging only) | `hello` |
| `hello` | ← | final | `app` (`AppInfo`), `protocol` | — |
| `status` | → | request | `web?` | `status` |
| `status` | ← | final | `app`, `remembered` | — |
| `choose` | → | request | `web?`, `filter? {algorithms?}` (non-empty) | `choose.result` |
| `choose.result` | ← | final | `certificates` (≥ 1) | — |
| `sign.begin` | → | request | `web?`, `hash`, `algorithms?` (non-empty), `certificate?` (fingerprint) | `sign.result` |
| `sign.need_digest` | ← | event | `seq`, `certificate`, `hash`, `algorithm` | — |
| `sign.digest` | → | continuation | `seq`, `digest` (Base64) | — |
| `sign.result` | ← | final | `certificate`, `hash`, `algorithm`, `signature` (Base64) | — |
| `cancel` | → | continuation | — | the open request ends with `error` `Aborted` |
| `diagnostics.open` | → | request | `tab?` | `done` |
| `done` | ← | final | — | — |
| `error` | ← | final | `code`, `message`, `details?` | — |

`web` is required over native messaging for `status`, `choose` and
`sign.begin`, and refused over `websign connect` (§6).

### 4.1 `status`

Never opens a window. `remembered` tells whether `choose` will answer without
one.

```json
→ {"v":1,"id":"1","type":"status","web":{"origin":"https://app.example","topOrigin":"https://app.example"}}
← {"v":1,"id":"1","type":"status","app":{…},"remembered":false}
```

### 4.2 `choose` (`certificates()`)

Never returns the machine's list (D2). A remembered caller gets, without a
window, the certificates it already used that are still present and usable
(most recent first); if none remains, the window opens. Anyone else gets the
confirmation window in *choose* mode; the result holds the one certificate
chosen.

```json
→ {"v":1,"id":"2","type":"choose","web":{…},"filter":{"algorithms":["ECDSA","RSASSA-PKCS1-v1_5"]}}
← {"v":1,"id":"2","type":"choose.result","certificates":[{
     "der":"MIIF…","chain":["MIIG…"],"fingerprint":"256adb9a…",
     "displayName":"Ana Beatriz Souza","issuerName":"AC SOLUTI Multipla v5",
     "notBefore":1741000000,"notAfter":1792000000,
     "key":{"type":"RSA","bits":2048},
     "algorithms":["RSASSA-PKCS1-v1_5","RSASSA-PSS"],
     "profile":{"icpBrasil":"A3","keyStorage":"hardware"}}]}
```

### 4.3 `sign.begin` … `sign.result`

See §5 for the state machine.

```json
→ {"v":1,"id":"3","type":"sign.begin","web":{…},"hash":"SHA-256"}
← {"v":1,"id":"3","type":"sign.need_digest","seq":1,"hash":"SHA-256",
   "algorithm":"RSASSA-PKCS1-v1_5","certificate":{…}}
→ {"v":1,"id":"3","type":"sign.digest","seq":1,"digest":"n4bQgYhMfWWaL+qgxVrQFaO/TxsrC4Is0V1sFbDwCgg="}
   … the person clicks Sign, enters the PIN …
← {"v":1,"id":"3","type":"sign.result","hash":"SHA-256","algorithm":"RSASSA-PKCS1-v1_5",
   "certificate":{…},"signature":"Qm9…"}
```

- `algorithm` is chosen by the app: the first entry of `algorithms` (default
  `ECDSA, RSASSA-PKCS1-v1_5, RSASSA-PSS`) that the key supports **and** its key
  store can produce (e.g. RSASSA-PSS is absent through legacy CAPI). No match →
  the certificate is shown disabled ("Not compatible with this request").
- RSASSA-PSS always uses MGF1 with the same hash and a salt as long as the
  digest (what CNG, CryptoTokenKit and PKCS#11 do; proven in the kit).
- `signature`: RSA → the modulus-length block; ECDSA → raw `r‖s`, each half
  left-padded to the curve size (the app converts DER from macOS).
- Before replying, the app verifies the signature against the certificate
  with `websign_core::verify` (all curves but brainpoolP512r1). A failure is
  `DriverFailure` — a driver returning garbage must not reach a PAdES file.

### 4.4 `cancel`

```json
→ {"v":1,"id":"3","type":"cancel"}
← {"v":1,"id":"3","type":"error","code":"Aborted","message":"the caller cancelled the request"}
```

The SDK sends it on `AbortSignal` and when `prepare` throws; the extension
sends it when the requesting tab closes or navigates (the window shows
"{site} cancelled the request").

### 4.5 `diagnostics.open`

Starts the diagnostics window in a **separate process** (so it outlives the
connection) and replies `done`. Only the extension popup and desktop clients
send it; the extension never forwards it from a page.

## 5. The sign flow

```
             ┌─────────┐ front of queue ┌─────────┐ listed ┌───────────┐
 sign.begin ─▶ Queued  ├───────────────▶│ Listing ├───────▶│ Selecting │◀──────────────┐
             └─────────┘                └─────────┘        └─────┬─────┘               │
                                                  release (auto if remembered,         │
                                                  else "Continue" — D11)               │
                                                                 ▼                     │ person switches
                                       need_digest(seq=n) ┌────────────────────┐       │ certificate
                                     ◀────────────────────┤ AwaitingDigest(n)  ├───────┤ (seq n+1,
                                       digest(seq=n) ────▶└─────────┬──────────┘       │  need_digest again)
                                                                    ▼                  │
                                                           ┌────────────────┐          │
                                                           │   Ready(n)     ├──────────┘
                                                           └───────┬────────┘
                                                        Sign (armed, PIN)
                                                                   ▼
                                                           ┌────────────────┐  PinIncorrect → Ready
                                                           │   Signing      ├──────────────────────
                                                           └───────┬────────┘
                                                                   ▼
                                                           sign.result (final)

 any state ── cancel / timeout / disconnect / Cancel button / fatal error ──▶ error (final)
```

Rules:

1. **One window per signature** (D1): the certificate is chosen in the window;
   the digest is asked for afterwards, because PAdES/CAdES put the certificate
   inside the signed attributes (`signing-certificate-v2`).
2. **Release of the certificate (D11).** `sign.need_digest` discloses the
   certificate to the caller. For a *remembered* caller the preselected
   certificate is released at once (the person already consented to "which
   certificate I use"). For anyone else it is released only when the person
   clicks **Continue** — otherwise any site could call `sign()` and cancel to
   harvest the holder's name and CPF, defeating D2.
3. **Switching certificate** issues a new `need_digest` with `seq + 1`; a
   digest for an older `seq` is ignored silently (it may be in flight). The
   code card shows "Preparing the document…" until the new digest arrives, and
   the Sign button re-arms.
4. **Digest length** must equal the hash length (32/48/64) → else
   `InvalidRequest` ends the request (the window shows an internal error; it
   is a site bug).
5. **PIN errors stay inside the window**: `PinIncorrect` returns to `Ready`
   with the field cleared; only the final outcome reaches the caller. Closing
   the window reports the last blocking condition (`docs/ux.md` §15).
6. **Timeouts**: the caller has `DIGEST_TIMEOUT` (60 s) to answer each
   `need_digest`; the person has `DECISION_TIMEOUT` (300 s) from the moment
   the request is on screen. Both end with `Timeout`.
7. **Disconnect** (browser closed the port, desktop stdin closed): every open
   request ends; windows show "{site} cancelled the request" and close.
8. **Queue**: one request on screen per process; up to 10 wait (FIFO), one
   more → `Busy`. `status` and remembered `choose` bypass the queue.

`choose` follows the same shape without digest: `Queued → Listing → Choosing →
choose.result`.

## 6. Caller identity

The window shows who asks from facts the app itself established — never text
from the caller (P1, R4).

| Transport | Identity | How it is established | Consent key |
|---|---|---|---|
| Native messaging | page origin, top-level origin, browser | the extension reads `MessageSender.url`/`origin` and the tab URL; the browser already verified the extension ID against the manifest (`allowed_origins`/`allowed_extensions`); the app re-checks the launch origin against `project.toml` IDs and re-validates the origin (`https`, or `http` only for localhost) | canonical origin (`https://app.example.com`) |
| `websign connect` / `sign` / `choose` | executable path, product name, code signer | the app reads its **parent process** before the first frame (Windows: Toolhelp + `QueryFullProcessImageNameW` + Authenticode; macOS: `proc_pidpath` + code signature; Linux: `/proc/<ppid>/exe`) | `app:<signer>` for signed programs, `path:<executable>` otherwise |

Consequences:

- A desktop caller cannot claim to be a website: `web` is refused on
  `websign connect`.
- A malicious local program can start `websign connect`, but the window names
  *that* program, and it is not remembered unless the person ticks
  "Remember this program". Unsigned programs are shown with an "Unverified
  program" warning.
- Local malware running as the user can already do most things the user can;
  the confirmation window and the PIN are the line we hold
  ([security.md](security.md)).

## 7. Errors

`error.code` values are stable strings (`websign_protocol::ErrorCode`), shared
with the SDK's `WebSignError.code` and `docs/ux.md` §15:

| Code | Raised by | When |
|---|---|---|
| `ExtensionMissing` | SDK | no announcement within 1 s |
| `AppMissing` | extension, client libs | `connectNative`/spawn failed |
| `AppOutdated` | extension, app | app < `MIN_APP_VERSION`, or protocol ranges |
| `ExtensionOutdated` | extension | app requires a newer protocol than the extension speaks |
| `ClientOutdated` | app | a desktop client speaks only older protocols |
| `InsecureOrigin` | extension (app as defense in depth) | not a secure context |
| `Aborted` | app | `cancel` received |
| `UserCancelled` | app | Cancel/Esc/close with nothing blocking |
| `Timeout` | app | decision or digest timeout |
| `NoCertificates` | app | closed with an empty list |
| `CertificateUnavailable` | app | chosen certificate vanished |
| `CertificateNotValid` | app | requested certificate expired/not yet valid |
| `InvalidRequest` | app, extension | malformed message, wrong digest length, bad sequence |
| `UnsupportedAlgorithm` | app | key/driver cannot produce the algorithm |
| `PinIncorrect` | app | only surfaces through `websign sign` in unattended tests |
| `PinLocked` | app | PIN blocked |
| `TokenRemoved` | app | token left while signing |
| `DriverFailure` | app | other key store failure; `details.native` has the code |
| `Busy` | app | queue full |
| `Internal` | app | a bug |

`message` is English, for developers, and never contains personal data.

## 8. Limits and timeouts

All in `websign_protocol::limits`. Size limits a single frame decides
(origin, `hello` texts, chain) are enforced by the parser; the others by the
app or the extension:

| Limit | Value | Why |
|---|---|---|
| Incoming frame | 1 MiB | every request is tiny; bounds allocation |
| Outgoing frame | 1 MiB | Chrome/Firefox drop hosts that exceed it; an oversized reply becomes `Internal` |
| Request id | 64 chars | safe to log and echo |
| Origin | 512 bytes | real origins are short |
| Client name / versions in `hello` | 64 bytes | informational only |
| Open requests per connection | 16 | bounds memory per peer |
| Queue | 10 waiting | `docs/ux.md` §4.11 |
| `hello` | 5 s after start | a process nobody talks to exits |
| Digest | 60 s per `need_digest` | the site's `prepare` hangs → `Timeout` |
| Decision | 300 s on screen | `docs/ux.md` §4.11 |
| Desktop idle | 300 s without requests | a forgotten `websign connect` exits |
| Extension idle close | 60 s | keeps drivers loaded between signatures, closes after |
| App response (extension) | 3 s for `hello` | popup "The app didn't respond" |
| Chain | 8 certificates | real chains are 2–4 |

## 9. Page messages

The page never reaches the app; it posts objects to its own window and the
content script relays them ([web-api.md](web-api.md) §3):

```js
// SDK → content script
{ source: "websign-page", kind: "discover" }
{ source: "websign-page", kind: "request", id: "p1", message: { type: "sign.begin", hash: "SHA-256" } }
// content script → SDK
{ source: "websign-extension", kind: "announce", extension: { version: "1.4.2", browser: "chrome" },
  protocols: { min: 1, max: 1 } }
{ source: "websign-extension", kind: "message", id: "p1", message: { type: "sign.need_digest", … } }
```

Page requests are a subset of the catalog without `web`, `hello` or
`diagnostics.open` (types `PageRequest`/`PageReply` in `websign_protocol::page`).
The extension maps `(tab, frame, page id)` to a connection-wide id, adds `web`
from the browser, and answers `status` itself (its own version plus the app's).

## 10. Evolution rules

1. Change the Rust types in `websign-protocol`, bump `PROTOCOL_VERSION`, keep
   the previous version parseable (a `v1` module per retired version).
2. `cargo xtask gen`; commit the generated TypeScript.
3. Add the new version's vectors to `SPEC.md` and the contract tests.
4. Never rename or reuse an error code or a message type; add new ones.
5. The extension and client libraries ship support for the new version before
   the app raises its `min`.
