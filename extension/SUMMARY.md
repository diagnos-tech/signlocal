# extension

- `src/` — sources of the extension
- `test/` — Vitest tests of the extension modules
- `README.md` — what this component is, how to build and test it, where its contract lives
- `SPEC.md` — the behavior contract: rules, errors, edge cases and test vectors (source of truth for blind TDD)
- `package.json` — extension package manifest and WXT scripts
- `tsconfig.json` — type-checking settings on top of WXT's generated config
- `wxt.config.ts` — WXT build configuration: one source, four targets (chrome, edge, firefox, safari).
