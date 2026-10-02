# extension/src/background

- `app-error.ts` — A failure to reach or talk to the app, already classified with a protocol code.
- `badge.ts` — The toolbar badge: "!" whenever the app is missing, outdated or not answering.
- `browser.ts` — Which browser hosts the extension, for `hello` (refines `shared/browser-name.ts` with async APIs).
- `connection.ts` — The native messaging port: opened on demand, `hello` first, kept open while used, closed after EXTENSION_IDLE_CLOSE (60 s) without traffic.
- `context.ts` — Who is asking, from the browser's MessageSender (tab, frame, secure web context).
- `native-port.ts` — The port the connection talks through: `connectNative`, or the Safari relay session when running in Safari.
- `origin.ts` — The requesting origins, from the browser's MessageSender (never from the payload).
- `popup-handler.ts` — Answers the toolbar popup: "is the app there?" and "open diagnostics".
- `relay-handler.ts` — Handles what content scripts send: page requests to route and "this document is going away" notices.
- `requests.ts` — The open-request table keyed by `<tab>.<frame>.<pageId>`, continuations and cancellation (tab, navigation, document).
- `router.ts` — Routes page requests to the app over the shared port and replies back to the right frame.
- `rpc.ts` — One request, one answer, over the shared connection (for the extension's own asks).
- `safari-port.ts` — A port-shaped Safari relay session: open, ordered sends, one long poll with backoff, close (`safari/SPEC.md` §3).
- `safari-relay.ts` — The appex relay contract: request shapes, limits, a strict reply reader and error-code mapping.
- `tabs.ts` — Tab lifecycle: a closed tab or a top-level navigation to another origin ends the requests made under it.
