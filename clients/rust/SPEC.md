# websign-client — specification

Rust twin of `@websign/desktop` ([`clients/node/SPEC.md`](../node/SPEC.md));
blocking API over `std::process`.

## 1. `find_executable`

Same search order as the Node client §1.

## 2. `Client`

- `connect_with`: spawn `websign connect` (stdin/stdout piped, stderr null;
  `CREATE_NO_WINDOW` on Windows); `hello` at `v = ProtocolRange::CURRENT.max`
  with `client {name: "websign-client", version: CARGO_PKG_VERSION}` unless
  overridden and `protocols = ProtocolRange::CURRENT`. The answer is parsed
  with `parse_hello_reply(frame, v)` (protocol SPEC §5.1: the app's `hello`,
  or an `error` at our `v`) within 10 s. Outcomes: spawn failure, exit
  before the answer → `AppMissing`; no answer → `App { Timeout }`; `error` →
  `App { code }`; no common version per `negotiate(app.protocols, CURRENT)`
  → `App { ClientOutdated | AppOutdated }`; `protocol` other than the
  negotiated one, unparseable frame, other id → `Connection`.
- Frames with `websign_protocol::framing` (a reader thread feeds a channel of
  16 frames, so a flooding app cannot grow memory); messages with
  `parse_app_message(frame, Some(negotiated))` and `to_json`. Request ids are
  `n1`, `n2`, …; one request open at a time. A reply for another id, an
  unexpected message type, a broken pipe → `Connection`, after which the
  client refuses every call.
- `sign`: `sign.begin`; for each `need_digest`: a `hash` other than the
  requested one → `cancel`, `Connection`; call `prepare(&certificate,
  algorithm)`; `Err(e)` → `cancel`, `Prepare(e)`; digest length ≠
  `hash.digest_len()` → `cancel`, `App { InvalidRequest }`; else
  `sign.digest`. Final `sign.result` → `Ok`; `error` → `App`. After a
  `cancel` the client waits at most 5 s in total for the request's final
  message, else the connection is abandoned (`Connection` on the next call).
- `Drop`: close stdin, wait up to 5 s, then kill; never blocks longer.
- Errors and `Debug` never contain the executable path, digests or
  certificate bodies (`Certificate`'s own `Debug` is the protocol crate's).
