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
- License: GPL-3.0-or-later.
