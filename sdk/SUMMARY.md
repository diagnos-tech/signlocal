# sdk

- `src/` — sources of `@websign/sdk` (entries: main, `/messages`, `/testing`)
- `test/` — Vitest tests of the SDK
- `scripts/` — size budget check of the built package (`bun run size`)
- `tools/` — API reference (TypeDoc) and package checks (publint, are-the-types-wrong); own lockfile
- `LICENSE` — Apache-2.0 text
- `README.md` — quickstart, errors, testing utilities, development scripts
- `SPEC.md` — the behavior contract: rules, errors, edge cases and test vectors (source of truth for blind TDD)
- `package.json` — package manifest: exports `.`, `./messages` and `./testing`, zero dependencies
- `tsconfig.build.json` — build settings (sources only, emits dist/)
- `tsconfig.json` — type-checking settings (sources and tests)
