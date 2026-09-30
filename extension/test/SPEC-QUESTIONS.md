# Questions the spec leaves open

Each item is pinned in the tests with `// SPEC:` where an assumption was needed.

1. Content-script to background message shape is unspecified. Assumed `runtime.sendMessage({id, message})` and replies
   delivered through `runtime.onMessage` as `{kind: "message", id, message}` (relay.test.ts).
2. Who answers `discover` (SPEC 3.2): `announce()` takes no arguments, so the wiring lives in the entrypoint and is not unit tested.
   `announce()` is synchronous, so how it learns the browser name is open (test only requires a valid `BrowserName`).
3. How `connect()` fails: assumed a rejection with `{code: "AppMissing" | "Internal", message}` (SPEC 2.1). The router
   is assumed to turn any rejection into a page `error` (plain `Error` -> some error) and to answer `status` as a
   `PageStatus` without `app` when the host is missing.
4. `route()` has no unsubscribe/teardown API, so tab removal and navigation are assumed to be observed by the router
   itself through `browser.tabs.onRemoved`/`onUpdated` (registered at import or on first `route`).
   Navigation is assumed to compare `changeInfo.url`'s origin against the sender's `topOrigin`.
5. "One native port per request" in the task text vs "one native port per background lifetime" in SPEC 2.1: tests follow
   the SPEC (one `connectNative` shared by concurrent and later callers, one native id per request).
6. Outdated app and `status`: unclear whether the app is still asked for `remembered`; tests assert only `appOutdated: true` and `app`.
7. `AppOutdated` probe in `PopupFacts` carries no versions, so `popupState` cannot fill `installed`; only `kind` is asserted.
8. Unknown fields in a page request (D12): validation may strip or refuse; tests accept both but never a forwarded `web`/`origin`.
9. `digest` validation checks only text length per SPEC 2.4; whether non-Base64 characters of the right length are refused is untested.
   Empty `algorithms` arrays are untested (SPEC says "at most 3", the catalog says "never empty").
10. Relay strings limit (256) vs nested objects: `filter.algorithms` is an array, "primitive fields only" cannot be literal. Untested.
11. Idle close "without open requests": the connection does not know about requests, so only traffic-based idle is tested.
12. Popup view rendering (`render`) is not tested: no DOM library is a dependency. i18n is tested at the catalog level
    (keys per state, placeholders, all locales); adding `happy-dom` would allow asserting `getMessage` keys and the `!` badge.
