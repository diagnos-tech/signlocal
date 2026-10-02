# @websign/sdk — specification

Surface and usage: [`docs/architecture/web-api.md`](../docs/architecture/web-api.md).
Wire: [`protocol.md`](../docs/architecture/protocol.md) §9. Tests run in
Vitest with a fake `window` whose `postMessage` is driven by a fake content
script (`test/helpers`).

## 1. Rules

- Zero runtime dependencies; ES2022; no globals written; tree-shakeable;
  published ESM with explicit `.js` import paths.
- Every rejection is a `WebSignError` whose `code` is in the protocol
  catalog; `status()` never rejects. Invalid options reject the returned
  promise (never a synchronous throw) before anything is posted.
- Nothing is sent before the extension announced itself.
- Globals (`window`, `location`, `navigator`) are read at each use, never
  captured at import. Without a `window` (server-side rendering) the SDK
  imports cleanly, `status()` reports nothing installed and the other calls
  reject `ExtensionMissing`.
- No console output from the main entry and `/messages`, ever (`/testing`
  warns on purpose, §14).
- Every `WebSignError` carries `hint` (the next step, English, for developers;
  one per code, `src/hints.ts`) and `docsUrl` =
  `HOMEPAGE + "developers.html#error-" + code`; `site/developers.html` has an
  anchor for every code (tested).

## 2. `channel` (internal, not exported from the package)

- On import, listen for `message` events where `event.source === window`,
  `event.origin === location.origin`, `data.source === "websign-extension"`,
  a known `kind` and the fields the SDK acts on (announce: `extension
  {version, browser}` strings, `protocols {min, max}` integers with
  `1 ≤ min ≤ max`; `sign.need_digest`: integer `seq ≥ 1`, object
  `certificate`, string `hash`/`algorithm`; `error`: string `code` and
  `message`). Anything else, including the SDK's own looped-back
  `websign-page` posts, is dropped silently. Unknown extra fields in a valid
  frame are tolerated: strictness (D12) lives in the extension and the app.
- Every post targets `location.origin`, never `"*"`, and only `window`
  itself: nothing is posted to `parent`, `top` or other frames.
- `discover()`: if an `announce` was seen, resolve with it; else post
  `{source:"websign-page", kind:"discover"}` and wait `DISCOVERY_TIMEOUT_MS`
  (1000); resolve `null` on timeout. Concurrent callers share one discovery.
  The result, `null` included, is cached: later calls neither wait nor post
  until an `announce` replaces it (and fires `onChange`).
- `send(request, onReply, signal)`: id = `"p" + counter + "." + 6 random
  base36 chars`; post `{source, kind:"request", id, message}`; deliver every
  reply with that id to `onReply(reply, id)`; `done` resolves after a final
  reply (`status`, `choose.result`, `sign.result`, `error`). `signal` abort →
  post a `cancel` request with the same id and reject with `Aborted` (do not
  wait). An already aborted signal rejects without posting.

## 3. Protocol version

The SDK speaks page protocol 1. After discovery, `certificates()` and
`sign()` refuse an announcement whose range excludes 1, posting nothing:

- `min > 1` → `ClientOutdated`, details `{installed: "1", required: min}`:
  the SDK is too old, the site must update `@websign/sdk`. (The catalog's
  `ClientOutdated` is "a client speaks only older protocols"; the SDK is a
  client of the extension. `errorText` has no user text for it: the person
  cannot fix it.)
- `max < 1` → `ExtensionOutdated` (unreachable while 1 is the lowest
  version; kept for SDKs that drop version 1).

`status()` reports such an extension as installed, the app as not
installed, `ready: false`, `problem: "ClientOutdated"` (or
`"ExtensionOutdated"`).

## 4. `convert`

- `toBase64`/`fromBase64`: RFC 4648 standard alphabet, padded, strict (same
  rules as the protocol crate §7) through `atob`/`btoa` (present in windows,
  workers and Node): a text is accepted only when re-encoding its bytes gives
  it back, which refuses whitespace, URL-safe letters, missing padding and
  stray bits. Only replies are decoded, so a failure is `Internal`.
- `toCertificate`: Base64 → `Uint8Array`, Unix seconds → `Date`, other fields
  copied; `profile.eidas.types` copied.

## 5. Replies

- `error` → `WebSignError(code, message, details)`; a code outside the
  catalog becomes `Internal`. `AppMissing`/`AppOutdated` also notify
  `onChange` subscribers.
- A final reply of the wrong kind (e.g. `choose.result` to a sign), or one
  the SDK cannot convert (missing fields, bad Base64) → `Internal`: a bug in
  the extension or app, nothing the site can fix.

## 6. `status()`

`discover()`; null → `{extension:{installed:false}, app:{installed:false,
outdated:false}, remembered:false, ready:false}`. Incompatible protocol →
§3. Else send `status`; reply `status` (`PageStatus`) → map; `error
AppMissing` → `app.installed=false`; other errors → `app.installed=true,
outdated = code === "AppOutdated"`. `problem` is the first of
`ExtensionMissing`, the protocol problem (§3), `AppMissing`, `AppOutdated`
that applies, absent when `ready`; `ready = !problem`. The extension answers `status` itself,
so no reply within 10 s (the extension needs up to 8 s + 1.5 s for an app's
first start) means it hung: abort (posting `cancel`) and report the
app as not installed. `ready = extension.installed && app.installed &&
!app.outdated`.

## 7. `certificates(options)`

Validate `algorithm` (below) → `connect` (§3; `ExtensionMissing` when
nobody answers) → send `choose` with `filter.algorithms` only when given.
Reply `choose.result` → certificates; an empty list → `NoCertificates` (the
app reports an empty choice as that error). `signal` as in `send`, also
while discovering.

Algorithm option (both calls): one name or an array, deduplicated keeping
order. Unknown or lowercase names, non-strings, `null` and an empty array →
`InvalidRequest` (an empty list must not silently mean "any").

## 8. `sign(options)`

1. Validate: options an object; `hash` one of the three; `algorithm` as §7;
   `certificate` a `Certificate` or its fingerprint (exactly 64 lowercase hex
   digits, as `certificates()` returns it); `prepare` a function → else
   `InvalidRequest`, nothing posted.
2. `signal` already aborted → `Aborted`, nothing posted (not even
   `discover`). Aborted while discovering → `Aborted`, no request posted.
3. `connect` (§3).
4. Send `sign.begin` with `hash`, and `algorithms` / `certificate`
   (fingerprint only; the certificate body never leaves the page) only when
   given — omitted, not `undefined`.
5. On each `sign.need_digest`:
   - a `seq` not above the last one seen is ignored;
   - `hash` ≠ the request's, or `algorithm` outside the requested list (all
     three when none was given) → `cancel` + `InvalidRequest`, before
     `prepare` (the protocol's code for a broken signing sequence);
   - a certificate that cannot be converted → `cancel` + `Internal`;
   - call `prepare(certificate, {hash, algorithm})` (await). Its result
     must be a `Uint8Array` (only the view's bytes are sent) or an
     `ArrayBuffer` of exactly the digest length, else `cancel` +
     `InvalidRequest` "Digest is N bytes; SHA-256 requires 32." (or "must
     return a Uint8Array or an ArrayBuffer"); `prepare` throws/rejects →
     `cancel` + `Aborted` with the original error as `cause` and a generic
     message (site errors may quote the document);
   - otherwise send `sign.digest {seq, digest}` under the request id.
   - Once a newer `need_digest` arrived, or the request ended (final reply,
     abort, earlier failure), an older `prepare`'s result and failure are
     both dropped.
6. `sign.result` → resolve (`toCertificate`, signature decoded, `digest` =
   a copy of the last digest sent, i.e. the one the app signed); `error` →
   reject; late replies after the SDK cancelled are ignored.

Types: `sign<H, A>(options: SignOptions<H, A>): Promise<SignResult<H, A>>`,
so `prepare`'s context and the result carry the literal hash and algorithm
union the caller passed. Bytes out are `Uint8Array<ArrayBuffer>` (accepted by
WebCrypto as is); bytes in are `Uint8Array | ArrayBuffer`.

## 9. `installUrl()`

Browser from the last announcement, else from `navigator.userAgentData`/UA
(never throws, also without `navigator`): Chrome/Chromium/Brave/Opera/Vivaldi
→ `https://chromewebstore.google.com/detail/<CHROME_WEB_STORE_ID>` when set;
Edge → `https://microsoftedge.microsoft.com/addons/detail/<EDGE_ADDONS_ID>`
when set; Firefox → `https://addons.mozilla.org/firefox/addon/<FIREFOX_AMO_SLUG>/`
when set (an unpublished slug can be claimed by anyone, so it is never
guessed); otherwise, and while an ID is empty → `HOMEPAGE + "download.html"`. Constants come
from the generated `project.ts`. Synchronous; posts nothing.

## 10. `onChange(listener)`

Calls `listener(await status())` after every `announce` seen after
subscribing and whenever a request fails with `AppMissing`/`AppOutdated`,
debounced: a burst within 250 ms gives one call. Not called at subscription.
Returns an idempotent unsubscribe.

## 11. `fingerprint(digest)`

The verification code of `websign-protocol` SPEC §8, same vectors. Takes a
`Uint8Array` (its view only) or an `ArrayBuffer`. Fewer than 8 bytes, or not
bytes → throws `WebSignError(InvalidRequest)` (the Rust API returns
`None`; a synchronous function has no promise to reject).

## 12. `@websign/sdk/messages`

`errorText(error, locale?)`: `error` is a code, a `WebSignError` or any
`{code, details?}`; `locale` defaults to `navigator.language`, else `en`.
Closest locale (`pt` → pt-BR, `es-MX` → es, …, else en) from the generated
`messages.gen.ts`; `{installed}`/`{required}` filled from `details` when
present, else left. `undefined` for `undefined` (so `status().problem` passes
as is) and for `InvalidRequest`, `PinIncorrect`, `ClientOutdated` (site bugs
or internal).

`isWebSignError(error, ...codes)` (main entry): `instanceof WebSignError`
and, when codes are given, one of them; narrows `code` in TypeScript.

## 13. Size

A consumer bundle of every export of the built main entry (Bun, minified) is
< 5 KB gzip: `bun run build && bun run size` (exits 1 over budget; for CI).
`/messages` and `/testing` are reported, not budgeted.

## 14. `@websign/sdk/testing`

`installFakeWebSign(options?)` answers the SDK's page frames on `window` as
the content script, extension and app would (tests: `test/testing/`):

- Frames arrive as `message` events whose data has `source: "websign-page"`;
  answers are dispatched as `message` events with `source` = the window and
  `origin` = its origin (not `postMessage`: some test DOMs drop both), one
  microtask later or after `latencyMs`. It announces at install and on
  `discover`, except in `extension-missing`.
- Certificates: real X.509 v3 DER, self-issued with fixed public test keys
  (EC P-256; RSA-2048 with PKCS#1 v1.5 and PSS); signatures over the digest
  computed with BigInt (RFC 6979 nonces, PSS salt derived from the digest),
  so certificates and signatures are identical on every run.
- The person picks `choose()` (default the first usable certificate); a
  preselected fingerprint it lacks → `CertificateUnavailable`; none usable →
  `NoCertificates`; outside validity → `CertificateNotValid`;
  `failNext(code)` fails the next choose/sign once; `switchDuringNextSign`
  sends `need_digest` seq 2 with another certificate after the first digest.
- Scenarios: `app-missing`/`app-outdated`/`extension-outdated` answer status
  accordingly and fail choose/sign with that code (`details` 0.9.0/1.0.0);
  `sdk-outdated` announces protocol 2.
- Guard: refuses origins other than localhost, 127.x, ::1, `*.localhost`,
  `.test`, `.example`, `.invalid`, `.local`, `file:`/`about:` unless
  `allowAnyOrigin`; `console.warn` at every install and once if a real
  extension also answers.
