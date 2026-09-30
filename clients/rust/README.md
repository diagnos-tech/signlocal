# websign-client

Call the WebeSign app from a Rust program: `Client::connect()`, then
`certificates()` or `sign(options, prepare)`. Starts `websign connect` on
demand; the person confirms every signature in the app's window, which names
your program.

- API guide: [`docs/architecture/desktop-api.md`](../../docs/architecture/desktop-api.md) §7.
- Contract: [`SPEC.md`](SPEC.md).
- License: **Apache-2.0** ([`LICENSE`](LICENSE)); depends only on the
  Apache-2.0 workspace crates `websign-protocol` and `websign-project`.

## Build and test

```sh
cargo test -p websign-client
```

The tests run against `examples/fake_websign`, a fake `websign connect`
(one scenario per misbehavior); no installed app is needed. The
`WEBSIGN_EXECUTABLE` environment variable overrides where the app is
looked up (`find_executable()`).
