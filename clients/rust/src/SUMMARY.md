# clients/rust/src

- `client.rs` — One connection to the app: the simple requests, request ids and cancellation.
- `error.rs` — What can go wrong, with the protocol's stable codes.
- `handshake.rs` — Starting the app, `hello` and version negotiation.
- `lib.rs` — Call the WebeSign app from a Rust program.
- `locate.rs` — Finding the `websign` executable.
- `locate/` — unit tests of the executable search.
- `options.rs` — What the caller asks for: connect options, sign options (algorithm set, preselection) and the `prepare` context.
- `process.rs` — Spawning `websign connect` and reaping it within bounded waits.
- `reader.rs` — The thread that turns the child's stdout into a channel of frames.
- `session.rs` — The framed pipe to one child: which messages it expects, unusable after a failure, cleanup on drop.
- `sign.rs` — `sign.begin` and the digest exchange: the app's request checked, `prepare`, cancellation.
- `timing.rs` — The two waits the client bounds itself: the answer to `hello` and the app's exit.
