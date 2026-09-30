# websign-client — specification

Rust twin of `@websign/desktop` ([`clients/node/SPEC.md`](../node/SPEC.md));
blocking API over `std::process`.

## 1. `find_executable`

Same search order as the Node client §1.

## 2. `Client`

- `connect_with`: spawn `websign connect` (stdin/stdout piped, stderr null);
  `hello` with `client {name: "websign-client", version: CARGO_PKG_VERSION}`
  unless overridden; `protocols = ProtocolRange::CURRENT`; wait for the
  reply (10 s); failures → `AppMissing` (spawn) or `App { code }`.
- Frames with `websign_protocol::framing`; messages with
  `parse_app_message(frame, Some(negotiated))` and `to_json`.
- `sign`: `sign.begin`; for each `need_digest`, call `prepare(&certificate,
  algorithm)`; `Err(e)` → send `cancel`, return `Prepare(e)`; wrong length →
  send `cancel`, `App { code: InvalidRequest }`; send `sign.digest`; final
  `sign.result` → `Ok`; `error` → `App`.
- `Drop`: close stdin, wait up to 5 s, then kill.
