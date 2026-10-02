# safari/Tests

- `BundledHostTests.swift` — the real `websign` from a packaged app answers `hello` through the relay (skipped without it)
- `FrameTests.swift` — framing byte order, chunking, oversized headers
- `HostProcessTests.swift` — a stubborn host is killed; closing stdin ends a good one
- `RelayRequestTests.swift` — exact shapes, bounds, and the client types against the generated protocol
- `RelayTests.swift` — sessions end to end with fake hosts (`cat`, `sh`): echo, exit, broken framing, limits, profiles, idle
- `Support.swift` — fake hosts and synchronous helpers
