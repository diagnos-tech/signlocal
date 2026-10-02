# safari

- `.gitignore` — SwiftPM build folders
- `Extension/` — the appex entry point (SafariServices)
- `Package.swift` — SwiftPM manifest of the relay and its tests
- `README.md` — what this component is, how to build and test it, where its contract lives
- `Relay/` — platform-neutral relay between Safari's messages and a `websign` host process
- `SPEC.md` — the relay contract: messages, replies, host launch, limits
- `Tests/` — XCTest suite with fake hosts, plus the packaged-host check
