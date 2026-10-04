# Node command-line example

Signs the SHA-256 of a file with `@websign/desktop`. By default it talks to the
fake app from `@websign/desktop/testing`, so it runs anywhere (CI included);
`--app` uses the installed SignLocal app and opens its window.

```sh
# from the repository root, once
(cd clients/node && npm install && npm run build)
cd examples/desktop/node-cli && npm install

node sign-file.mjs README.md          # fake app: prints the "signature" (the digest itself)
node sign-file.mjs contract.pdf --app # the real app: pick a certificate, confirm with the PIN
npm test                              # runs both paths' error handling in CI
```

Exit codes: 0 signed, 1 failure (code, message, hint and docs link on stderr),
2 usage, 3 the person cancelled.

In your own project install the package instead of the `file:` path:
`npm install @websign/desktop`.
