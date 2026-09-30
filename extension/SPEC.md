# WebeSign extension — specification

Role: the only bridge between pages and the app. No UI but the toolbar popup
(`docs/ux.md` §9). Wire: [`protocol.md`](../docs/architecture/protocol.md);
page side: [`web-api.md`](../docs/architecture/web-api.md) §3. The kit's
extension (`docs/prototypes/kit/extension/`) is proven reference code for
§2.1 and §3.1.

## 1. Manifest

MV3 for every target; permission `nativeMessaging` only (no
`host_permissions`, no `web_accessible_resources`, no
`externally_connectable`); content script on `https://*/*`,
`http://localhost/*`, `http://*.localhost/*`, `http://127.0.0.1/*`,
`http://[::1]/*` (the loopback names §2.3 accepts, no more), all frames,
`document_start`; Firefox `browser_specific_settings.gecko.id` from
`project.toml`, minimum 121; `default_locale: en` with `_locales` generated
from `i18n/`; name, description and toolbar title are `__MSG_…__`.

- Channel `WEBSIGN_CHANNEL` (`build/manifest.ts`): `direct` (default; the
  release zip people load unpacked) and every dev-server build carry
  `project.toml`'s `dev_key` on Chromium targets, which pins the extension
  ID to `dev_id`, the only ID the native host manifest allows. `store`
  builds never carry it (the stores own the ID). Any other value fails the
  build.
- Production builds set `content_security_policy.extension_pages` to
  `default-src 'none'; script-src 'self'; style-src 'self'; img-src 'self';
  object-src 'none'; base-uri 'none'; form-action 'none'`: no remote code, no
  inline script, no network from extension pages.
- `SUMMARY.md` files are not shipped.

## 2. Background

### 2.1 Connection

- One native port per background lifetime, opened on first need and shared
  by every tab: `connect(reason = "page")` →
  `runtime.connectNative(NATIVE_HOST)`; first message `hello` with `client
  {name:"websign-extension", version: manifest.version}`, `protocols
  {min:1,max:1}`, `browser {name, version, reason}` (`reason` of the call
  that opened the port: `page`, `popup`, `installed`; the protocol's
  `startup` is unused). Concurrent and later callers get the same connection until it closes; the next call
  after a close or failed attempt opens a new port.
  **Why shared, not per request:** starting the app costs a process launch
  and a driver load, the app's queue and PKCS#11 session (PIN cache, D5)
  live per connection, and hello runs once. Isolation between tabs does not
  come from ports but from ids (§2.2): the tab and frame in every native id
  come from the browser, so a page can neither name, continue, cancel nor
  receive another tab's or frame's request.
- `connect()` rejects with an `AppError {code, message, details?}`:
  `connectNative` throwing or the port closing before any message →
  `AppMissing` ("host not found"); no `hello` reply in time → `AppMissing`
  too (a host that never speaks is, to the person, a broken install with
  the same remedy); a `hello` reply with another id, a first message that
  is not `hello`, or an unknown protocol → `Internal`; an
  `error` instead of `hello` → its code (`ClientOutdated` becomes
  `ExtensionOutdated`) and `details` (so `AppOutdated` carries
  `{installed, required}`). The router answers any other rejection with
  `Internal`.
- Hello reply: `app.version` older than `MIN_APP_VERSION` → every page
  request gets `AppOutdated` with `details {installed, required}`.
- `hello` deadline: 8 s until the app has answered once in this
  background's life (a first launch after install or update may be scanned
  by SmartScreen, Gatekeeper or an antivirus), then `APP_RESPONSE_TIMEOUT`
  (3 s) from websign-protocol, via the generated `shared/limits.gen.ts`.
- Idle close after 60 s without traffic and without open requests (a
  request is open from the message that starts it until `choose.result`,
  `sign.result`, `status`, `done` or `error` with its id).
- Port disconnect: every open request gets `AppMissing` (never heard) or
  `Internal` (heard) — "the app closed the connection".
- `onInstalled` with reason `install` or `update`: connect with reason
  `installed`, then let it idle out (this is how the app learns about every
  browser with the extension and checks its registration). Not on
  `runtime.onStartup`: starting the app at every browser launch would cost a
  process and a driver load for a check that only changes on install or
  update; otherwise the app starts when a page or the popup asks.

### 2.2 Router

- Native ids: `"<tabId>.<frameId>.<pageId>"` (≤ 64 chars; page ids are
  1–40 of `[A-Za-z0-9._:-]`, else `InvalidRequest`). Tab and frame ids are
  integers, so the prefix is unambiguous; the extension's own requests use
  `ext.<n>`, which no tab id can produce. An app message reaches only the
  entry with exactly its id; messages for unknown ids are dropped.
- `sign.digest` and `cancel` are continuations: sent under the native id of
  the sender's own open request, silently dropped when there is none (so
  another tab or frame cannot feed or cancel it). A second request with an
  open id → `InvalidRequest`; more than 16 open → `Busy`.
- A page request is forwarded with `web` from §2.3; replies are posted back
  to the same tab and frame (`tabs.sendMessage` with `frameId`), tagged with
  the requesting document's token (§3.1).
- An app older than `MIN_APP_VERSION` is never sent page requests: `choose`
  and `sign.begin` get `AppOutdated {installed, required}`; `status` is
  answered by the extension alone.
- `status` is answered as `PageStatus`: extension version and browser, the
  hello reply's `app` (or none), `appOutdated`, and `remembered` from the
  app's `status` reply (`false` when outdated, missing, silent for 1.5 s or
  closed). It never fails.
- Cancellation (`cancel` sent for, and the entry forgotten, of every
  matching open request), with listeners registered synchronously at
  background start (`watchTabs()`):
  - `tabs.onRemoved` → every request of that tab;
  - `tabs.onUpdated` with `status: "loading"` and a `url` whose origin
    differs from the request's `topOrigin` (an unparseable URL counts as
    different) → those requests of that tab (`webNavigation` is not
    requested; the content-script matches give access to the URL);
  - the content script's `websign-gone` on `pagehide` (reload, navigation,
    close, back-forward cache) → the requests of that tab, frame and
    document only, so a late goodbye never cancels the page that replaced
    it.

### 2.3 Origin

`webContext(frameUrl, tabUrl)` from `MessageSender.url` (frame) and
`sender.tab.url` (top): both must be secure contexts (`https:`, or `http:`
for `localhost`, `*.localhost`, `127.0.0.1`, `[::1]`: exactly the loopback
hosts the content script matches, §1); return
`{origin, topOrigin}` as `new URL(x).origin`; else `null` → the page gets
`InsecureOrigin`. Requests from non-tab senders are ignored.

Permission note for the implementer: `sender.url` is always available;
`sender.tab.url` needs host permission for the tab. MV3 content-script
matches grant it on Chromium; if a target withholds it, ask the top frame's
content script for `location.origin` (`tabs.sendMessage` with `frameId: 0`)
— never trust a value the requesting frame supplies about its parent.

### 2.4 Validation

`validatePageRequest` (`src/shared/validate.ts`, run by the content script
and again by the background) rebuilds a `PageRequest` field by field from
the value's own enumerable fields: known `type`; `hash` in the 3 names;
`algorithms` a non-empty array of ≤ 3 known names; `certificate` 64
lowercase hex; `seq` integer 1..2³¹; `digest` standard Base64
(`[A-Za-z0-9+/]`, `=` padding only at the end) of 44, 64 or 88 characters
(32, 48 or 64 bytes; the app checks canonical padding); `filter` an object
whose only field is `algorithms`, like `algorithms`. Unknown fields at any
level are refused, not stripped (D12). Anything else → `null` →
`InvalidRequest`.

### 2.5 Browser detection

`src/shared/browser-name.ts`, synchronous: Chromium
`navigator.userAgentData.brands` (`Microsoft Edge` → edge, `Brave` → brave,
`Opera` → opera, `Vivaldi` → vivaldi, `Google Chrome` → chrome, else
chromium); else the UA (`Firefox/` → firefox; `Safari/` without
Chrome/Chromium/Edg → safari; else other). The background refines it for
`hello` with Firefox `runtime.getBrowserInfo()` and UA-CH
`fullVersionList`; major.minor version.

## 3. Content script

### 3.1 Relay

Accept `window` messages with `event.source === window`, `event.origin ===
location.origin`, `data` a plain object with `source === "websign-page"`,
`kind` `discover` (→ §3.2) or `request` with a valid page id (§2.2). The
request's `message` is rebuilt with `validatePageRequest` (§2.4); a refused
one is answered locally with `InvalidRequest` and never sent. Other fields
of `data` are never read. Every post to `window` uses `targetOrigin =
location.origin`, never `"*"`.

Messages between content script and background (`src/shared/runtime-messages.ts`):

| Direction | Shape |
|---|---|
| content → background | `{kind: "websign-request", document, id, message}` |
| content → background | `{kind: "websign-gone", document}` on `pagehide` |
| background → content | `{kind: "websign-reply", document, id, message: PageReply}` |
| background → top frame | `{kind: "websign-origin"}` → answered with `location.origin` |

`document` is a random UUID per content-script instance (one per
document). The background ignores messages from other extensions, from
senders without a tab, or without a valid `document`; it reads tab, frame
and URLs only from `MessageSender`. The content script accepts only
messages whose `sender.id` is its own extension, posts a reply only when its
`document` is its own (a reply that races a navigation is dropped, never
shown to the next page), and answers `websign-origin` only in the top
frame. The popup talks to the background with `{kind: "websign-popup", op:
"probe" | "diagnostics"}` and is recognized by having no tab and an
extension-page URL.

### 3.2 Announce

Post `announce {extension {version, browser}, protocols}` at start, on
`load`, and in answer to each `discover`. `announce()` is synchronous (the
SDK waits at most 1 s): `browser` is `currentBrowser().name` (§2.5).

## 4. Popup

`popupState(facts)`: platform not win/mac/linux → `unsupported`; probe null
→ `checking` (rendered only after 150 ms); ok and version ≥ min → `ready`;
ok and older → `outdated {installed, required: min}`; `AppMissing` →
`missing`; `AppOutdated` → `outdated` with the probe's `details.installed`
(else `""`, shown as "?") and `details.required` (else min); other codes →
`error {code}`. The probe is `{ok: true, appVersion} | {ok: false, code,
details?: {installed?, required?}}`.

View per `docs/ux.md` §9 with `browser.i18n.getMessage`; the card is a
`role="status"` section inside a `aria-live="polite"` root; icons are
`aria-hidden`; the primary button takes focus; links open in a new tab with
`rel="noopener noreferrer"` and point to the site's English pages
(`HOMEPAGE` + `download.html`, `privacy.html`, and `test/` for "Activate the
app", which follows the extension's announcement live). "Open diagnostics"
sends `diagnostics.open` and closes the popup (a failure re-checks
instead); "Try again" probes again. The toolbar badge shows `!` in
missing/outdated/error and clears in ready. Total popup size (popup.html
and the scripts and styles it loads) < 15 KB, checked by `bun run size`.

## 5. `isOlder(version, minimum)`

Numeric dotted versions of 1–3 parts, missing parts = 0; any non-numeric
input is "older". Vectors: `("0.9.9","1.0.0")` → true; `("1.0","1.0.0")` →
false; `("1.10.0","1.9.9")` → false; `("1.0.0-beta","1.0.0")` → true;
`("","0.0.1")` → true.
