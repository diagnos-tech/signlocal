# Rust command-line example

Signs the SHA-256 of a file with `websign-client`. By default it talks to the
crate's `FakeApp` (feature `testing`), so it runs anywhere, CI included;
`--app` uses the installed SignLocal app and opens its window.

```sh
cd examples/desktop/rust-cli
cargo run -- Cargo.toml          # fake app: prints the "signature" (the digest itself)
cargo run -- contract.pdf --app  # the real app: pick a certificate, confirm with the PIN
cargo test                       # runs the CLI against the fake
```

Exit codes: 0 signed, 1 failure (code, message, hint and docs link on stderr),
2 usage, 3 the person cancelled.

It is a standalone workspace (its own `Cargo.lock`) and depends on the client
by path; in your own project use `websign-client = "0.1"`, and the `testing`
feature only under `[dev-dependencies]`.
