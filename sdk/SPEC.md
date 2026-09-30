# @websign/sdk — specification

Surface and usage: [`docs/architecture/web-api.md`](../docs/architecture/web-api.md).
Wire: [`protocol.md`](../docs/architecture/protocol.md) §9. Tests run in
Vitest with a fake `window` whose `postMessage` is driven by a fake content
script.

## 1. Rules

- Zero runtime dependencies; ES2022; no globals written; tree-shakeable.
- Every rejection is a `WebSignError`; `status()` never rejects.
- Nothing is sent before the extension announced itself.

## 2. `channel`

- On import, listen for `message` events where `event.source === window`,
  `event.origin === location.origin`, `data.source === "websign-extension"`.
- `discover()`: if an `announce` was seen, resolve with it; else post
  `{source:"websign-page", kind:"discover"}` and wait `DISCOVERY_TIMEOUT_MS`
  (1000); resolve `null` on timeout. Cache the result; a later `announce`
  replaces it (and fires `onChange`).
- `send(request, onReply, signal)`: id = `"p" + counter + "." + 6 random
  base36 chars`; post `{source, kind:"request", id, message}`; deliver every
  `message` with that id to `onReply`; `done` resolves after a final reply
  (`status`, `choose.result`, `sign.result`, `error`). `signal` abort → post a
  `cancel` request with the same id and reject with `Aborted` (do not wait).

## 3. `convert`

- `toBase64`/`fromBase64`: RFC 4648 standard alphabet, padded, strict (same
  rules as the protocol crate §7); implemented without `atob` (works in
  workers).
- `toCertificate`: Base64 → `Uint8Array`, Unix seconds → `Date`, other fields
  copied; `profile.eidas.types` copied.

## 4. `status()`

`discover()`; null → `{extension:{installed:false}, app:{installed:false,
outdated:false}, remembered:false, ready:false}`. Else send `status`; reply
`status` (`PageStatus`) → map; `error AppMissing` → `app.installed=false`;
other errors → `app.installed=true, outdated = code === "AppOutdated"`.
`ready = extension.installed && app.installed && !app.outdated`.

## 5. `certificates(options)`

`discover()` null → `ExtensionMissing`. `algorithm` normalized to an array
(deduplicated, order kept) → `filter.algorithms`. Reply `choose.result` →
certificates; `error` → `WebSignError(code, message, details)`.

## 6. `sign(options)`

1. Validate: `hash` one of the three; `algorithm` values valid; `prepare` a
   function → else `InvalidRequest` (no message sent).
2. `discover()` null → `ExtensionMissing`.
3. Send `sign.begin` with `hash`, `algorithms`, `certificate` (fingerprint
   string or `certificate.fingerprint`).
4. On each `sign.need_digest`: call `prepare(certificate, {hash, algorithm})`
   (await); coerce `ArrayBuffer` → `Uint8Array`; length ≠ digest length →
   send `cancel` and reject `InvalidRequest` "Digest is N bytes; SHA-256
   requires 32."; `prepare` throws → send `cancel`, reject `Aborted` with the
   original error as `cause`. Otherwise send `sign.digest {seq, digest}`.
   A newer `need_digest` while an older `prepare` is running: the older
   result is discarded.
5. `sign.result` → resolve (`toCertificate`, signature decoded); `error` →
   reject.

## 7. `installUrl()`

Browser from the last announcement, else from `navigator.userAgentData`/UA:
Chrome/Brave/Opera/Vivaldi → Chrome Web Store URL when
`CHROME_WEB_STORE_ID` is set; Edge → Edge Add-ons when `EDGE_ADDONS_ID` is
set; Firefox → `https://addons.mozilla.org/firefox/addon/websign/`; otherwise
and while IDs are empty → `HOMEPAGE + "download"`. Constants come from the
generated `project.ts`.

## 8. `onChange(listener)`

Calls `listener(await status())` on every `announce` after the first
subscription and whenever a request fails with `AppMissing`/`AppOutdated`
(debounced 250 ms). Returns an idempotent unsubscribe.

## 9. `fingerprint(digest)`

The verification code of `websign-protocol` SPEC §8, same vectors.

## 10. `@websign/sdk/messages`

`errorText(code, locale)`: closest locale (`pt` → pt-BR, `es-MX` → es, …,
else en) from the generated `messages.gen.ts`; `undefined` for
`InvalidRequest`, `PinIncorrect`, `ClientOutdated`.

## 11. Size

`dist/index.js` minified + gzip < 5 KB (CI).
