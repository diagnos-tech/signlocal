# examples/desktop/node-cli

- `README.md` — how to run the example and what its exit codes mean
- `package.json` — the example's manifest (depends on `@websign/desktop` by path)
- `sign-file.mjs` — the CLI: SHA-256 of a file, signed through the fake or the real app
- `sign-file.test.mjs` — runs the CLI against the fake app and with the app missing
