# Architecture

The contracts of SignLocal: what each part does, how the parts talk, and why.
Implementation tracks work from these pages and from the `SPEC.md` next to
each component. The UX specification is [`docs/ux.md`](../ux.md); the plan and
the decisions are in [`docs/plan.md`](../plan.md).

| Page | Read it for |
|---|---|
| [overview.md](overview.md) | components, crate graph, processes, threads, data flow — nothing runs permanently |
| [protocol.md](protocol.md) | framing, envelope, versioning, the message catalog with examples, the sign-flow state machine, caller identity, error codes, limits |
| [web-api.md](web-api.md) | `@websign/sdk`: TypeScript surface, page ↔ extension messages, discovery |
| [desktop-api.md](desktop-api.md) | the CLI with flags, JSON output and exit codes; `websign connect`; client libraries; desktop caller identity |
| [security.md](security.md) | threat model, mitigations and where each is enforced |
| [i18n.md](i18n.md) | locale files, key generation, plurals, the extension and SDK outputs, CI checks |
| [testing.md](testing.md) | blind TDD, contract tests, UI tests, renderer, e2e matrix with screenshots, CI gates |
| [packaging-and-release.md](packaging-and-release.md) | channels, artifacts, install locations, one-line installers, unsigned-build notice, release workflow |
| [repository-layout.md](repository-layout.md) | folders, conventions, the `SUMMARY.md` format |
| [compatibility.md](compatibility.md) | algorithms × key stores × devices × browsers × OSes; brainpool; why EdDSA is out |
