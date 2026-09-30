# websign-client

Call the WebeSign app from a Rust program: `Client::connect()`, then
`certificates()` or `sign(options, prepare)`. Starts `websign connect` on
demand; the person confirms every signature in the app's window, which names
your program.

- API guide: [`docs/architecture/desktop-api.md`](../../docs/architecture/desktop-api.md) §7.
- Contract: [`SPEC.md`](SPEC.md).
- License: **Apache-2.0** ([`LICENSE`](LICENSE)); depends only on the
  Apache-2.0 protocol crates.
