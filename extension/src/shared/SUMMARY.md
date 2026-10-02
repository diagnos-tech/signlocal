# extension/src/shared

- `browser-name.ts` — The browser's name, synchronously, from UA-CH brands or the user agent (announcements, `hello`).
- `limits.gen.ts` — Protocol version, message sources, lengths and timers generated from websign-protocol (`cargo xtask gen`).
- `limits.ts` — Numbers and names both sides of the extension must agree on: the generated ones plus the extension's own timeouts and id rules.
- `runtime-messages.ts` — Messages between the content script, the background and the popup.
- `validate.ts` — `validatePageRequest`: rebuilds a page request field by field or refuses it (content script and background).
- `version.ts` — Numeric dotted versions ("1.10.2"); anything else compares as older.
