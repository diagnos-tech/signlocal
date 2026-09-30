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
`InvalidRequest` and exits with status 1. Defense in depth: the browser
already enforced the manifest.

## 2. `session::Session::accept`

1. `parse_client_message(frame, negotiated)`; a `ParseError` becomes a
   `Rejection` with the same id; `close = true` when there is no id or when
   the session has not negotiated yet.
2. `hello`:
   - second `hello` → `InvalidRequest`, `close = false`;
   - `validate` (§2.2); a refusal closes (`close = true`): nothing about a
     peer whose first frame breaks the transport rules is trusted;
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
   `InvalidRequest`); desktop → the transport's caller. The in-flight limit
   (16) is above what the engine can reach (1 active + 10 queued, windowless
   requests end at once), so it is tested on `Session` directly.

### 2.1 Replies before `hello`

Every error sent before negotiation (a refused `hello`, a first frame that
is not `hello`, an unknown extension) carries `v =
websign_protocol::refusal_version(frame)` of the frame it refuses (protocol
`SPEC.md` §5.1), so a client whose range does not overlap the app's can
still read `AppOutdated`/`ClientOutdated`.

### 2.2 `validate(transport, message)`

| Message | Native messaging | Desktop |
|---|---|---|
| `hello` | `browser` required | `browser` refused |
| `status`, `choose`, `sign.begin` | `web` required | `web` refused |
| `diagnostics.open` | allowed | allowed |
| `sign.digest`, `cancel` | allowed | allowed |

Refusals are `InvalidRequest` naming the field.

## 3. `caller`

`Caller::web` formats both origins with `format_origin` (both must be secure:
a frame inside an insecure page is not a secure context); `top = None` when
their canonical forms are equal. `consent_key` and `can_remember` as
documented in the code.

## 4. Sign flow (`flow::sign`)

States and transitions: `protocol.md` §5. The flow never sees the caller or
the queue: the engine passes a `Presentation` (caller view, `can_remember`,
queue position) to `activate(now, &presentation)` and a fresh
`ListContext` to every `on_listed(snapshot, context)`. The engine builds the
context from the request (`algorithms`, `certificate`) and the stores (the
certificate this caller used last, recent use anywhere), so the preselected
certificate is the list rules' own: `sign.begin.certificate` when usable,
else the one last used by this caller, else the first usable row. Effects per
transition:

| Event | State → | Effects (in order) |
|---|---|---|
| `activate(now)` | Queued → Listing | `Ui(Open{…})`, `Keys(List{refresh:false})`; deadline = now + 300 s |
| `on_listed` | Listing → Selecting | `Ui(Certificates{…})`; if remembered and a certificate is preselected: → AwaitingDigest(1) with `Send(NeedDigest{seq:1})`, `Ui(DigestPending)`; if nothing usable: stay Selecting (window shows Empty) |
| `on_listed` again | same state | `Ui(Certificates{…})` only |
| `Ui(Selected fp)` | Selecting/AwaitingDigest/Ready → | remembered: AwaitingDigest(seq+1), `Send(NeedDigest)`, `Ui(DigestPending)`; new: Selecting{fp} |
| `Ui(Continue fp)` | Selecting → AwaitingDigest(seq+1) | `Send(NeedDigest)`, `Ui(DigestPending)` |
| (any release above) | | then `Keys(Chain)` once per certificate |
| `on_digest(seq, d)` stale seq (issued, not current, or not awaiting) | unchanged | none |
| `on_digest` seq never issued | → Done | `Send(Error InvalidRequest)`, `Ui(Failed Internal)` |
| `on_digest` wrong length | → Done | `Send(Error InvalidRequest)`, `Ui(Failed Internal)` |
| `on_digest` ok | AwaitingDigest → Ready | `Ui(DigestReady{code})` |
| `Ui(Sign{fp, via, pin, remember})` in Ready with the same fp | → Signing(tag) | `Keys(Sign{…})`, `Ui(Signing)` |
| `Keys(Signed ok)` | → Done | verify (§4.1); `Keys(Chain)` result used if already known else empty chain; `RecordConsent`; `Send(SignResult)`; `Ui(Finished Signed)` |
| `Keys(Signed WrongPin)` | → Ready | `Ui(Failed PinIncorrect{flags from the pin_state of the path tried})` |
| `Keys(Signed PinLocked)` | → Selecting | `Ui(Failed PinLocked)`, `RecordError` |
| `Keys(Signed Cancelled)` (OS dialog cancelled) | → Ready | `Ui(Failed …)` none: back to Ready silently |
| `Keys(Signed TokenRemoved/NotFound)` | → Selecting | `Ui(Failed TokenRemoved/CertificateUnavailable)` |
| `Keys(Signed Unsupported)` | → Selecting | `Ui(Failed UnsupportedAlgorithm)` |
| `Keys(Signed Native/Other)` | → Ready | `Ui(Failed DriverFailure{driver of the path tried, alternate: via 0 and an alternate exists})`, `RecordError` |
| `Ui(Cancel code)` | → Done | `Send(Error code)`, `Ui(Finished …)` |
| `end(code)` (abort, timeout) | → Done | `Send(Error code)`; `Ui(Finished Timeout)` for a timeout, else `Ui(Finished Aborted)` (only when on screen) |
| `disconnected()` | → Done | `Ui(Finished SiteCancelled)` when on screen; nothing is sent |

`Ui(Sign{via: n})` signs through `KeyRef { path: n }` with the window's PIN:
the window shows its PIN field for a driver path (`websign-ui-model`
`SPEC.md` §2.2.2), so the PIN reaches `C_Login` there.

Window events for a fingerprint that is not listed, or whose row is
disabled, are ignored without a reply, as are events in any other state
than the table's. `KeyCommand::Sign.parent_window` is `None` from the flow;
the engine fills it with `ConfirmUi::parent_window()`.

Digest timeout: 60 s after each `NeedDigest` without the matching digest →
`end(Timeout)`. Decision timeout: the deadline → `end(Timeout)`. The
deadline does not apply while `Signing` (an OS PIN dialog or a slow token;
the caller can still `cancel`); a signing that fails after it ends at the
next tick.

Chain: asked (`Keys(Chain)`) when a certificate is first released, without
delaying `sign.need_digest`, whose certificate carries the chain only when
it is already known. `sign.result` carries the chain when the answer arrived
before the signature (the key store serves commands in order, so the real
worker always answers first), else an empty one. The chain is unsigned CMS
data, so a caller can add it after the digest.

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

Same `activate(now, &presentation)` / `on_listed(snapshot, context)` as §4.

- Remembered with at least one still-present usable certificate:
  `answers_without_window() == true`; `activate` → `Keys(List)`;
  `on_listed` → `Keys(Chain)` for each remembered present one; when the last
  chain arrives (`on_keys`) → `Send(ChooseResult{those, most recent first,
  each with its chain})`, Done. None present → the flow returns to `Queued`
  without remembered certificates and the engine queues it for the window
  like a new caller.
- Otherwise: queue, `Ui(Open{mode: Choose})`, `Ui(Certificates)`; `Ui(Choose
  {fp, remember})` for a usable row → `Keys(Chain)`; its answer →
  `RecordConsent`, `Send(ChooseResult{[that one]})`, `Ui(Finished Chosen)`.
  Further `Choose` events while the chain is read are ignored.

## 6. Queue

`push`: first → active (`Ok(true)`); then up to 10 waiting (`Ok(false)`);
11th waiting → `Err(Busy)`. `remove(active)` promotes the oldest waiting.
`position()` = `(1, 1 + waiting.len())` while active, `(0, 0)` idle.
Windowless requests never enter the queue. Whenever the line grows or
shrinks under the active request, the engine sends `Ui(Queue{active,
position})`; a request reaching the screen gets its position in `Open`.

## 7. Stores

Files in `data_dir()`: `consent.json`, `usage.json`, `connections.json`,
`errors.json`, `settings.json`, each `{"version": 1, …}`.

- `JsonFile::read`: missing → default; unreadable JSON → rename to
  `<name>.corrupt` (replacing an older one), log, default.
- `JsonFile::update`: `File::lock` on `<name>.lock` (exclusive, blocking up
  to 2 s, then `Io`), read, change, write `<name>.tmp`, `fsync`, rename.
  On Unix the folder is created `0700` and every file `0600` (they say which
  sites get certificates without asking); on Windows the profile ACL applies.
- Consent: `remember` inserts or refreshes; `record_use` only touches
  existing records; certificates list most-recent-first, at most 20.
- Usage: most-recent-first, at most 100 fingerprints.
- Errors: newest last, at most 20.
- `RecordConsent{remember, fp}`: `remember` (when the caller `can_remember`)
  or `record_use`; always `usage.record(fp)` — usage is anonymous. A failed
  self-check records nothing.
- A native `hello` records a `ConnectionRecord` (browser, versions); a
  desktop `hello` none.
- Store failures never fail a request: the engine logs the failure kind (never
  the path) and continues without persistence.

## 8. Engine scenarios (with fake ports)

Each is a test: events in, assert frames out, UI commands, key commands.

1. `hello` negotiation ok; version mismatch both ways (error at the hello's
   `v`); `status` before `hello` → error + close; second `hello` rejected,
   connection kept. Every close before negotiation is `Control::Exit(1)`.
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
   `Finished SiteCancelled`, no frames, `Control::Exit(0)` (`Broken` →
   `Exit(1)`).
10. Decision timeout (manual clock +300 s) → `Timeout`; digest timeout
    (+60 s) → `Timeout`; none while `Signing`.
11. Queue: 1 active + 10 waiting accepted, 12th → `Busy`; finishing the first
    opens the second (`Open` with position (1, 10)).
12. Remembered `choose` answers without UI (after the chains); revoked in
    another process between two `choose` → the second opens the window.
13. Device event while a request is on screen → `Keys(Invalidate)`,
    `Keys(List{refresh: true})`, then `Ui(Certificates)`; selection kept. A
    reader or card removal first sends `Keys(EndSessions)` (D5: a PIN is
    cached until its token leaves, and which token left is unknown). Nothing
    on screen → ignored.
14. `diagnostics.open` → launcher called, `done`; a failing launcher →
    `Internal`.
15. Desktop idle 300 s without requests → `Control::Exit(0)`; a native
    connection never idles out (the extension closes the port).
16. No `hello` within `HELLO_TIMEOUT` (5 s) → `Control::Exit(0)`.
17. Slow listing: `Keys(SlowListing{device})` while a `List` is unanswered
    and a request is on screen → `Ui(SlowListing{key, device})`; after the
    `Listed` (the notice crossed it) or with nothing on screen → nothing.
    "Scan again" starts a new listing.
18. `Ui(ViewCertificate{key, fp})` for the request on screen →
    `ConfirmUi::view_certificate(DER of fp from the last listing)`; an
    unlisted fingerprint or another key → nothing; nothing is sent.
19. Alternate path: primary Windows store (`System`), alternate driver
    (`App`): a native error on path 0 → `DriverFailure{driver: "Windows",
    alternate: true}`; `Sign{via: 1, pin}` → `Keys(Sign{path: 1, pin})`; a
    wrong PIN reports the driver's flags. End to end over pipes with a
    failing fake store and SoftHSM2 as the alternate, the window being the
    real `ConfirmModel` (`runtime/softhsm_tests/alternate.rs`).

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
  `pin_state` for the primary path and for each alternate `KeyPath`, algorithms from the key type and the store's capabilities)
  plus `possible`: a fresh `websign_devices::Snapshot::scan()` through
  `possible_devices` with the listed keys' `LinkedDevices` (reader names,
  token models, `unknown_links` when a hardware key has no `DeviceLink` or
  only a CryptoTokenKit one), keeping the confident entries as
  `PossibleCard`s; replies in command order. A listing still running after
  `SLOW_LISTING` (2 s) first posts `KeyReply::SlowListing{device}`: the
  `devices.json` model of the first known plugged-in token or card (from the
  same scan), else `None` — the key stores report no progress, and a token
  label or serial is never used.
- Neither the device monitor nor the hints database is wrapped in
  `catch_unwind`: `monitor::start` cannot fail (it retries on its own
  thread) and a bad embedded database is an error value; a panic in a
  helper thread ends that thread only, and the list then refreshes only on
  "Scan again".
- `serve(input, outbound, config, make_ui, launcher, stores, options)` is
  `serve_stdio` over any streams and stores (the SoftHSM2 test uses pipes).
- `ConfirmUi::view_certificate(der)` must not block (the Windows and macOS
  viewers are modal): the app hands the bytes to its UI thread. The default
  implementation has no viewer.
- The `testing` feature exposes fakes of every port for other crates' tests.
