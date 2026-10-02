# crates/websign-protocol/src/messages

- `control.rs` — Messages that steer a connection rather than sign.
- `hello.rs` — `hello`: the first message of every connection.
- `mod.rs` — The message catalog of protocol version 1.
- `sign.rs` — The signing exchange: `sign.begin` → (`sign.need_digest` ↔ `sign.digest`)+ → `sign.result`.
- `status.rs` — `status` and `choose`: questions that never sign anything.
