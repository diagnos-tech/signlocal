# crates/websign-host/src

- `engine/` — the event loop's parts: frames, effects, queueing, lifecycle, persistence
- `flow/` — per-request state machines
- `launch/` — tests of browser-launch detection
- `ports/` — what the engine needs from the outside, as traits
- `runtime/` — real threads and stdio around the engine
- `session/` — one connection's protocol state
- `store/` — small JSON stores: consent, usage, connections, errors, settings
- `testing/` — fakes of every port (feature `testing`) and a real certificate fixture
- `caller.rs` — Who is asking, established by the transport — never by the payload.
- `engine.rs` — The deterministic core loop of a host process.
- `launch.rs` — Recognizing that a browser (not a person) started this process.
- `lib.rs` — The session engine of the app: everything between a transport and the windows, with no OS or UI code of its own.
- `queue.rs` — One request on screen, the rest waiting (`docs/ux.md` §4.11).
