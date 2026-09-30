# extension/src/background

- `browser.ts` — Which browser hosts the extension (for `hello` and install links).
- `connection.ts` — The native messaging port: opened on demand, `hello` first, kept open while used, closed after EXTENSION_IDLE_CLOSE (60 s) without traffic.
- `origin.ts` — The requesting origins, from the browser's MessageSender (never from the payload).
- `router.ts` — Routes page requests to the app and replies back to the right frame.
- `validate.ts` — Validates page requests again in the background (the content script shares a process with the page and cannot be trusted): shape, types, sizes, hash names, digest Base64 length.
