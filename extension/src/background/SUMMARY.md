# extension/src/background

- `app-error.ts` — A failure to reach or talk to the app, already classified with a protocol code.
- `badge.ts` — The toolbar badge: "!" whenever the app is missing, outdated or not answering.
- `browser.ts` — Which browser hosts the extension (for `hello` and install links).
- `connection.ts` — The native messaging port: opened on demand, `hello` first, kept open while used, closed after EXTENSION_IDLE_CLOSE (60 s) without traffic.
- `context.ts` — Who is asking, from the browser's MessageSender (tab, frame, secure web context).
- `origin.ts` — The requesting origins, from the browser's MessageSender (never from the payload).
- `popup-handler.ts` — Answers the toolbar popup: "is the app there?" and "open diagnostics".
- `relay-handler.ts` — Handles what content scripts send: page requests to route and "this frame is going away" notices.
- `router.ts` — Routes page requests to the app and replies back to the right frame.
- `rpc.ts` — One request, one answer, over the shared connection (for the extension's own asks).
- `tabs.ts` — Tab lifecycle: a closed tab or a navigation to another origin ends the requests it made.
- `validate.ts` — Validates page requests again in the background (the content script shares a process with the page and cannot be trusted): shape, types, sizes, hash names, digest Base64 length.
