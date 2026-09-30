# examples/web

- `index.html` — landing page listing the examples
- `vanilla/` — single-file HTML example: status, sign, localized errors
- `react/` — React example with a copy-paste `useWebSign()` hook
- `vue/` — Vue example with a `useWebSign()` composable
- `pades/` — PDF signing (PAdES B-B) with pdf-lib and PKI.js, plus its OpenSSL-verified test
- `shared/` — installs the testing fake (unless `?real`) with its control panel; shared styles; Vue shim
- `vite.config.ts` — one dev server for every example; `@websign/sdk` aliased to `sdk/src`
- `tsconfig.json` — type-checking settings (same aliases)
- `package.json` — standalone package with its own lockfile (not a workspace member)
- `biome.json` — lint and format settings for this folder
- `README.md` — how to run, what each example shows, where the PDF library meets WebeSign
