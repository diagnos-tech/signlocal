# safari/Relay

- `BundledHost.swift` — the host command for a packaged appex: its own `websign` copy, browser-launch arguments
- `Frame.swift` — native messaging framing: encoder and a chunk-proof reader
- `HostProcess.swift` — one `websign` process on pipes; stop = close stdin, then terminate, then kill
- `Limits.swift` — every size and time bound of the relay, with the reason for each
- `Relay.swift` — sessions over Safari's one-reply-per-message native messaging: open, send, poll, close
- `RelayReply.swift` — the two reply shapes
- `RelayRequest.swift` — strict parsing of the background's messages; session IDs; wire error codes
- `Session.swift` — one host's pending output, bounded; its end
