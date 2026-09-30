# xtask

`cargo xtask <command>` — the repository's automation, so CI and people run
the same steps:

| Command | Does |
|---|---|
| `gen` | TypeScript types from `websign-protocol`, extension `_locales` and SDK messages from `i18n/`, project constants for TS |
| `check [summaries\|i18n\|generated\|release]` | repository invariants (`SUMMARY.md` coverage, locale keys, fresh generated files, no e2e marker in release builds) |
| `package --target <triple>` | release artifacts for one target |
| `screenshots --from <dir> --os <name>` | copies e2e PNGs into `docs/screenshots/<os>/` and indexes them |

Details: [`docs/architecture/testing.md`](../docs/architecture/testing.md),
[`repository-layout.md`](../docs/architecture/repository-layout.md).
