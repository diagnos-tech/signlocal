# websign-host

The session engine of a host process: browser-launch detection, the `hello`
handshake, per-transport caller rules, the sign and choose flows, the request
queue, consent and other small stores, timeouts, and the runtime that wires
stdio, the key store worker and the device monitor. The UI, the key stores,
the clock and process launching are ports, so every scenario runs in tests
with fakes.

- Wire: [`docs/architecture/protocol.md`](../../docs/architecture/protocol.md);
  processes: [`overview.md`](../../docs/architecture/overview.md).
- Contract and scenarios: [`SPEC.md`](SPEC.md). `launch` is promoted from the
  Phase-0 kit.
- Tests: `cargo test -p websign-host` runs every scenario over recording
  fakes (`tests/common`; the crate's own fakes are `websign_host::testing`,
  behind the `testing` feature), and serves a whole signature over pipes with
  the real key worker against a SoftHSM2 token (skipped when
  `softhsm2-util`, `pkcs11-tool` and `openssl` are missing;
  `WEBSIGN_REQUIRE_SOFTHSM=1` makes that a failure).
- License: GPL-3.0-or-later.
