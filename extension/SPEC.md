# WebeSign extension — specification

Role: the only bridge between pages and the app. No UI but the toolbar popup
(`docs/ux.md` §9). Wire: [`protocol.md`](../docs/architecture/protocol.md);
page side: [`web-api.md`](../docs/architecture/web-api.md) §3. The kit's
extension (`docs/prototypes/kit/extension/`) is proven reference code for
§2.1 and §3.1.

## 1. Manifest

MV3 for every target; permission `nativeMessaging` only; content script on
`https://*/*`, `http://localhost/*`, `http://127.0.0.1/*`, `http://[::1]/*`,
all frames, `document_start`; development builds carry `key` (pinned ID);
Firefox `browser_specific_settings.gecko.id` from `project.toml`, minimum
121; `default_locale: en` with `_locales` generated from `i18n/`.

## 2. Background

### 2.1 Connection

- One native port per background lifetime, opened on first need:
  `runtime.connectNative(NATIVE_HOST)`; first message `hello` with `client
  {name:"websign-extension", version: manifest.version}`, `protocols
  {min:1,max:1}`, `browser {name, version, reason}`.
- No `hello` reply within 3 s → `AppMissing` if the port disconnected
  without any message ("host not found"), else `Internal` ("the app did not
  respond"); used by the popup's `error` state.
- Hello reply: `app.version` older than `MIN_APP_VERSION` → every page
  request gets `AppOutdated` with `details {installed, required}`.
- Idle close after 60 s without traffic and without open requests.
- Port disconnect: every open request gets `AppMissing` (never heard) or
  `Internal` (heard) — "the app closed the connection".
- `runtime.onStartup` and `onInstalled`: connect with reason `startup` /
  `installed`, then let it idle out (this is how the app learns about every
  browser with the extension).

### 2.2 Router

- Native ids: `"<tabId>.<frameId>.<pageId>"` (≤ 64 chars; page ids longer
  than 40 are refused as `InvalidRequest`).
- A page request is forwarded with `web` from §2.3; replies are posted back
  to the same tab and frame (`tabs.sendMessage` with `frameId`).
- `status` is answered as `PageStatus`: extension version and browser, the
  hello reply's `app` (or none), `appOutdated`, and `remembered` from the
  app's `status` reply.
- `tabs.onRemoved` and top-level navigation (`webNavigation` is not
  requested; use `tabs.onUpdated` with `status: "loading"` and a changed URL
  origin) → `cancel` every open request of that tab.

### 2.3 Origin

`webContext(frameUrl, tabUrl)` from `MessageSender.url` (frame) and
`sender.tab.url` (top): both must be secure contexts (`https:`, or `http:`
for `localhost`, `*.localhost`, `127.0.0.0/8`, `[::1]`); return
`{origin, topOrigin}` as `new URL(x).origin`; else `null` → the page gets
`InsecureOrigin`. Requests from non-tab senders are ignored.

Permission note for the implementer: `sender.url` is always available;
`sender.tab.url` needs host permission for the tab. MV3 content-script
matches grant it on Chromium; if a target withholds it, ask the top frame's
content script for `location.origin` (`tabs.sendMessage` with `frameId: 0`)
— never trust a value the requesting frame supplies about its parent.

### 2.4 Validation

`validatePageRequest` rebuilds a `PageRequest` field by field: known `type`;
`hash` in the 3 names; `algorithms` an array of ≤ 3 known names;
`certificate` 64 lowercase hex; `seq` integer 1..2³¹; `digest` Base64 of 32,
48 or 64 bytes (length check on the text: 44, 64 or 88 characters);
`filter.algorithms` like `algorithms`. Anything else → `null` →
`InvalidRequest`.

### 2.5 Browser detection

Firefox `runtime.getBrowserInfo()`; Chromium `navigator.userAgentData.brands`
(`Google Chrome` → chrome, `Microsoft Edge` → edge, `Brave` → brave, `Opera`
→ opera, `Vivaldi` → vivaldi, else chromium); Safari by UA; major.minor
version.

## 3. Content script

### 3.1 Relay

Accept `window` messages with `event.source === window`, `event.origin ===
location.origin`, `data.source === "websign-page"`, `kind` `discover` or
`request`, `id` a string ≤ 40 chars; rebuild the request (primitive fields
only, strings ≤ 256 chars) and `runtime.sendMessage` it; post replies to
`window` with `targetOrigin = location.origin`.

### 3.2 Announce

Post `announce {extension {version, browser}, protocols}` at start, on
`load`, and in answer to each `discover`.

## 4. Popup

`popupState(facts)`: platform not win/mac/linux → `unsupported`; probe null
→ `checking` (rendered only after 150 ms); ok and version ≥ min → `ready`;
ok and older → `outdated`; `AppMissing` → `missing`; `AppOutdated` →
`outdated`; other codes → `error`. View per `docs/ux.md` §9 with
`browser.i18n.getMessage`; "Open diagnostics" sends `diagnostics.open` and
closes the popup; the toolbar badge shows `!` in missing/outdated/error. Total
popup size < 15 KB.

## 5. `isOlder(version, minimum)`

Numeric dotted versions of 1–3 parts, missing parts = 0; any non-numeric
input is "older". Vectors: `("0.9.9","1.0.0")` → true; `("1.0","1.0.0")` →
false; `("1.10.0","1.9.9")` → false; `("1.0.0-beta","1.0.0")` → true;
`("","0.0.1")` → true.
