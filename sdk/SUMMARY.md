# sdk

- `src/` — sources of `@websign/sdk`
- `test/` — Vitest tests of the SDK
- `LICENSE` — Apache-2.0 text
- `README.md` — what this component is, how to build and test it, where its contract lives
- `SPEC.md` — the behavior contract: rules, errors, edge cases and test vectors (source of truth for blind TDD)
- `package.json` — package manifest: exports `.` and `./messages`, zero dependencies
- `tsconfig.build.json` — build settings (sources only, emits dist/)
- `tsconfig.json` — type-checking settings (sources and tests)
