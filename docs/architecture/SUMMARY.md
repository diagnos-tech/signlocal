# docs/architecture

The contracts of WebeSign and the reasons behind them; implementation tracks work from these pages and the `SPEC.md` of each component.

- `README.md` — index of the pages and what to read each one for
- `compatibility.md` — algorithms × key stores × devices × browsers × OSes; brainpool; why EdDSA is out
- `desktop-api.md` — the CLI (flags, JSON output, exit codes), `websign connect`, client libraries, desktop caller identity and consent
- `i18n.md` — locale files, generated keys, plurals and dates, extension and SDK outputs, CI checks
- `overview.md` — components, crate graph, processes and threads, data flow, state on disk, channels
- `packaging-and-release.md` — channels, artifacts, install locations, one-line installers, unsigned-build notice, release workflow
- `protocol.md` — framing, envelope, versioning, message catalog with JSON examples, sign-flow state machine, caller identity, errors, limits
- `repository-layout.md` — folder map, conventions, deviations from the first sketch, the `SUMMARY.md` format
- `security.md` — assets, trust boundaries, threat table with mitigations and proofs, log rules
- `testing.md` — blind TDD, contract suites, UI tests, renderer evidence, e2e matrix with screenshots, CI gates
- `web-api.md` — `@websign/sdk` surface and types, page ↔ extension messages, discovery, behavior per call
