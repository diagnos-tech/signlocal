# clients/rust/src/testing

- `app.rs` — `FakeApp` and its builder: configuration, connection over in-process pipes, request log
- `mod.rs` — the `testing` feature's public surface and how to enable it
- `sample.rs` — the fictional certificate the fake offers by default
- `serve.rs` — the fake's side of the pipes: parses requests and answers like the app
