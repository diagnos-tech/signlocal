# websign-protocol

The wire contract between the app and everything that talks to it: message
types for native messaging and `websign connect`, page ↔ extension messages,
strict parsing, framing, error codes, limits, and the verification-code
algorithm. The TypeScript types of the SDK, the extension and the Node client
are generated from here (`cargo xtask gen`, feature `typescript`).

- Design and examples: [`docs/architecture/protocol.md`](../../docs/architecture/protocol.md).
- Contract and vectors: [`SPEC.md`](SPEC.md). `framing` and `base64` are
  promoted from the Phase-0 kit.
- Tests: `cargo test -p websign-protocol`.
- License: **Apache-2.0** (see [`LICENSE`](LICENSE)), so client libraries can
  depend on it; it depends on nothing GPL.
