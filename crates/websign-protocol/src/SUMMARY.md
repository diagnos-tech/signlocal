# crates/websign-protocol/src

- `envelope/` — the strict frame parser, its helpers and its tests
- `framing/` — tests of the framing
- `messages/` — the message catalog
- `types/` — value types messages are made of
- `base64.rs` — Standard Base64 (RFC 4648 §4, padded), the encoding the SDK uses for digests and signatures.
- `code.rs` — The verification code: a short, comparable rendering of a digest.
- `envelope.rs` — The envelope around every message and the strict parser.
- `error.rs` — Stable error codes and the error message.
- `framing.rs` — Native messaging wire format: a `u32` length in the machine's own byte order, followed by that many bytes of UTF-8 JSON.
- `id.rs` — Request identifiers.
- `lib.rs` — The SignLocal wire contract, shared by every party that talks to the app.
- `limits.rs` — Every size and time limit of the protocol, in one place.
- `page.rs` — Page ↔ extension messages (`window.postMessage`).
- `strict.rs` — Serde glue that makes the derived impls as strict as the protocol: objects only, no `null`.
- `version.rs` — Protocol versions and their negotiation.
