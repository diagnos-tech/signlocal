# probe/src/nm

Native messaging: running as the browser's host and registering with browsers. Test modules live in same-named subfolders.

- `backend.rs` — what the protocol handler needs from the machine's key sources, as a trait
- `base64.rs` — standard padded Base64 for digests and signatures
- `framing.rs` — wire format: `u32` length in native byte order followed by UTF-8 JSON
- `framing/` — tests for the framing
- `handler.rs` — turns one request frame into one reply, validating before any key source is touched
- `handler/` — tests for the handler
- `host.rs` — the message loop until the browser closes the port
- `host/` — tests for the loop
- `keystore_backend.rs` — the real backend: the CLI's key sources, opened once per process
- `keystore_backend/` — the backend's source listing, a test fixture, and tests
- `launch.rs` — recognizes that a browser, not a person, started the process
- `launch/` — tests for launch detection
- `log.rs` — diagnostic log of the host (no personal data)
- `log/` — tests for the log
- `mod.rs` — module overview
- `protocol.rs` — the JSON messages exchanged with the extension
- `protocol/` — tests for the protocol
- `register/` — writing and removing the host manifest for each browser
- `summary.rs` — certificate to wire summary for `list` replies
