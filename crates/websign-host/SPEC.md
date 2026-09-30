# websign-host — specification

The session engine. Wire: [`protocol.md`](../../docs/architecture/protocol.md);
processes and threads: [`overview.md`](../../docs/architecture/overview.md).
Everything here is testable with fake ports and a manual clock; only
`runtime` touches real threads and stdio.

## 1. `launch` (promoted from the kit, reviewed)

`parse_launch(args)`: Chromium family when `args[0]` is
`chrome-extension://<32 a–p>/` (optional `--parent-window=<decimal>`, zero
= none); Firefox when `args[0]` ends in `.json` and contains a path
separator and `args[1]` is a plausible ID (non-empty, ≤ 256, no leading `-`,
no whitespace or control characters); anything else → `None`. Promoted tests
in `src/launch/tests.rs`.

Additional rule for the product: the engine refuses a native-messaging launch
whose extension ID is not in `websign_project::chromium_extension_ids()` (or
equal to `FIREFOX_ID` for Firefox) — it answers the first frame with
`InvalidRequest` and exits. Defense in depth: the browser already enforced
the manifest.

## 2. `session::Session::accept`

1. `parse_client_message(frame, negotiated)`; a `ParseError` becomes a
   `Rejection` with the same id; `close = true` when there is no id or when
   the session has not negotiated yet.
2. `hello`:
   - second `hello` → `InvalidRequest`, `close = false`;
   - `validate` (§2.2);
   - `negotiate(app.protocols, hello.protocols)`; failure → Rejection with
     that code, `close = true`;
   - success: store `negotiated` and `browser`; `Accepted { caller: None }`.
3. Continuations (`sign.digest`, `cancel`): the id must be open
   (`lookup`), else — `cancel`: accepted and ignored (the request may have
   ended); `sign.digest`: `InvalidRequest`.
4. Requests: the id must **not** be open (`InvalidRequest` "duplicate id");
   at most `MAX_IN_FLIGHT_PER_CONNECTION` open (`Busy`); `validate`; the
   caller: native messaging → `Caller::web(web, browser)` (an
   `OriginError::Insecure` → `InsecureOrigin`, `Malformed` →
   `InvalidRequest`); desktop → the transport's caller.

### 2.2 `validate(transport, message)`

| Message | Native messaging | Desktop |
|---|---|---|
| `hello` | `browser` required | `browser` refused |
| `status`, `choose`, `sign.begin` | `web` required | `web` refused |
| `diagnostics.open` | allowed | allowed |
| `sign.digest`, `cancel` | allowed | allowed |

Refusals are `InvalidRequest` naming the field.

## 3. `caller`

`Caller::web` formats both origins with `format_origin`; `top = None` when
equal. `consent_key` and `can_remember` as documented in the code.

## 4. Sign flow (`flow::sign`)

States and transitions: `protocol.md` §5. Effects per transition:

| Event | State → | Effects (in order) |
|---|---|---|
| `activate(now)` | Queued → Listing | `Ui(Open{…})`, `Keys(List{refresh:false})`; deadline = now + 300 s |
| `on_listed` | Listing → Selecting | `Ui(Certificates{…})`; if remembered and a certificate is preselected: → AwaitingDigest(1) with `Send(NeedDigest{seq:1})`, `Ui(DigestPending)`; if nothing usable: stay Selecting (window shows Empty) |
| `on_listed` again | same state | `Ui(Certificates{…})` only |
| `Ui(Selected fp)` | Selecting/AwaitingDigest/Ready → | remembered: AwaitingDigest(seq+1), `Send(NeedDigest)`, `Ui(DigestPending)`; new: Selecting{fp} |
| `Ui(Continue fp)` | Selecting → AwaitingDigest(seq+1) | `Send(NeedDigest)`, `Ui(DigestPending)` |
| `on_digest(seq, d)` stale seq | unchanged | none |
| `on_digest` wrong length | → Done | `Send(Error InvalidRequest)`, `Ui(Failed Internal)` |
| `on_digest` ok | AwaitingDigest → Ready | `Ui(DigestReady{code})` |
| `Ui(Sign{fp, via, pin, remember})` in Ready with the same fp | → Signing(tag) | `Keys(Sign{…})`, `Ui(Signing)` |
| `Keys(Signed ok)` | → Done | verify (§4.1); `Keys(Chain)` result used if already known else empty chain; `RecordConsent`; `Send(SignResult)`; `Ui(Finished Signed)` |
| `Keys(Signed WrongPin)` | → Ready | `Ui(Failed PinIncorrect{flags from pin_state})` |
| `Keys(Signed PinLocked)` | → Selecting | `Ui(Failed PinLocked)`, `RecordError` |
| `Keys(Signed Cancelled)` (OS dialog cancelled) | → Ready | `Ui(Failed …)` none: back to Ready silently |
| `Keys(Signed TokenRemoved/NotFound)` | → Selecting | `Ui(Failed TokenRemoved/CertificateUnavailable)` |
| `Keys(Signed Unsupported)` | → Selecting | `Ui(Failed UnsupportedAlgorithm)` |
| `Keys(Signed Native/Other)` | → Ready | `Ui(Failed DriverFailure{alternate})`, `RecordError` |
| `Ui(Cancel code)` | → Done | `Send(Error code)`, `Ui(Finished …)` |
| `end(code)` (abort, timeout, disconnect) | → Done | `Send(Error code)` unless disconnected; `Ui(Finished Aborted/Timeout/SiteCancelled)` |

Digest timeout: 60 s after each `NeedDigest` without the matching digest →
`end(Timeout)`. Decision timeout: the deadline → `end(Timeout)`.

### 4.1 Self-check

Before `SignResult`, `websign_core::verify(cert, hash, algorithm, digest,
signature)` when `curve.has_verifier()` (RSA always). Failure →
`Ui(Failed DriverFailure{native:"signature did not verify"})`, state Ready,
`RecordError`; nothing is sent.

### 4.2 Algorithm choice

The first of `request.algorithms` (default `DEFAULT_PREFERENCE`) contained
in the candidate's `algorithms`. None → the certificate is disabled
(`Incompatible`) by the list rules, so it cannot be selected.

## 5. Choose flow (`flow::choose`)

- Remembered with at least one still-present usable certificate:
  `answers_without_window() == true`; `activate` → `Keys(List)`;
  `on_listed` → `Send(ChooseResult{remembered present ones, most recent
  first})`, Done. None present → behaves like a new caller.
- Otherwise: queue, `Ui(Open{mode: Choose})`, `Ui(Certificates)`; `Ui(Choose
  {fp, remember})` → `RecordConsent`, `Send(ChooseResult{[that one]})`,
  `Ui(Finished Chosen)`.

## 6. Queue

`push`: first → active (`Ok(true)`); then up to 10 waiting (`Ok(false)`);
11th waiting → `Err(Busy)`. `remove(active)` promotes the oldest waiting.
`position()` = `(1, 1 + waiting.len())` while active, `(0, 0)` idle.
Windowless requests never enter the queue.

## 7. Stores

Files in `data_dir()`: `consent.json`, `usage.json`, `connections.json`,
`errors.json`, `settings.json`, each `{"version": 1, …}`.

- `JsonFile::read`: missing → default; unreadable JSON → rename to
  `<name>.corrupt` (replacing an older one), log, default.
- `JsonFile::update`: `File::lock` on `<name>.lock` (exclusive, blocking up
  to 2 s, then `Io`), read, change, write `<name>.tmp`, `fsync`, rename.
- Consent: `remember` inserts or refreshes; `record_use` only touches
  existing records; certificates list most-recent-first, at most 20.
- Usage: most-recent-first, at most 100 fingerprints.
- Errors: newest last, at most 20.
- Store failures never fail a request: the engine logs and continues without
  persistence.

## 8. Engine scenarios (with fake ports)

Each is a test: events in, assert frames out, UI commands, key commands.

1. `hello` negotiation ok; version mismatch both ways; `status` before
   `hello` → error + close; second `hello` rejected.
2. Native messaging without `web` on `sign.begin` → `InvalidRequest`; desktop
   with `web` → `InvalidRequest`; `http://evil.example` → `InsecureOrigin`.
3. Remembered site: `sign.begin` → Open + List → Certificates → NeedDigest(1)
   → digest → DigestReady → Sign → Signed → verify ok → `sign.result`.
4. New site: no NeedDigest until `Continue`; `Cancel` before Continue → the
   caller got an error and **no certificate** (D11).
5. Switch certificate: NeedDigest(2); a late digest for seq 1 is ignored; seq
   2 digest → DigestReady.
6. Wrong digest length → `InvalidRequest`.
7. Wrong PIN then right PIN: one `sign.result`, no error frame.
8. Key store returns garbage → not verified → DriverFailure in UI, nothing sent.
9. `cancel` from the client → `Aborted`; client disconnect → UI
   `Finished SiteCancelled`, no frames.
10. Decision timeout (manual clock +300 s) → `Timeout`; digest timeout
    (+60 s) → `Timeout`.
11. Queue: 1 active + 10 waiting accepted, 12th → `Busy`; finishing the first
    opens the second (`Open` with position (1, 10)).
12. Remembered `choose` answers without UI; revoked → window.
13. Device event while Selecting → `Keys(Invalidate)`, `Keys(List{refresh:
    true})`, `Ui(Certificates)`; selection kept.
14. `diagnostics.open` → launcher called, `done`.
15. Desktop idle 300 s without requests → `Control::Exit(0)`.

## 9. Runtime

- `serve_stdio`: creates the channel; spawns the stdin reader, the key worker
  (`spawn_key_worker`), the device monitor, a 250 ms ticker; builds `Ports`;
  loops `Engine::handle` until `Exit`; returns the status.
- stdin reader: `framing::read_frame` in a loop → `Frame`; `Ok(None)` →
  `Closed`; errors → `Broken(reason)` and stop.
- `StdoutOutbound::send`: `to_json`; larger than 1 MiB → replaced by an
  `Internal` error for the same id (as the kit did); `write_frame`.
- Key worker: owns `KeystoreHub::new(options)`; `List` maps the inventory to
  `CertCandidate`s (device labels via `websign_devices::hints`, PIN mode via
  `pin_state`, algorithms from the key type and the store's capabilities);
  replies in command order.
