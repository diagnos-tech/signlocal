# xtask

`cargo xtask <command>` — the repository's automation, so CI and people run
the same steps. `cargo xtask --help` and `cargo xtask <command> --help` list
every option.

| Command | Does |
|---|---|
| `gen [--only ts\|locales\|messages\|project]` | TypeScript types from `websign-protocol` (sdk, extension, clients/node; imports use `.js` specifiers), extension `_locales` and SDK `messages.gen.ts` (typed by `ErrorCode`) from `i18n/`, `project.ts` from `project.toml` (the SDK's `src/project.ts` carries only store IDs and homepage) |
| `check [summaries\|i18n\|generated\|release] [--binary <websign>]` | repository invariants: `SUMMARY.md` coverage and component READMEs, locale keys, fresh generated files, lockstep versions, licenses, no e2e marker in the release binary |
| `package --target <triple>` | release artifacts for one target (implemented by the packaging track; fails with that message until then) |
| `screenshots --from <dir> --os <name>` | copies e2e PNGs into `docs/screenshots/<os>/` and rewrites its `index.md` and `SUMMARY.md` |

Design: each generator only builds a plan (files and their content);
`gen` writes it and `check generated` compares it with the tree, so the two
cannot disagree. Output is deterministic (sorted, `\n` line endings, no
timestamps), and a CRLF checkout on Windows counts as current. Every check reports all its problems, not the first.

Details: [`docs/architecture/testing.md`](../docs/architecture/testing.md),
[`repository-layout.md`](../docs/architecture/repository-layout.md).

Tests: `cargo test -p xtask` (temporary folders, no network). `gen` and
`check generated` run `cargo test -p websign-protocol --features typescript`.
