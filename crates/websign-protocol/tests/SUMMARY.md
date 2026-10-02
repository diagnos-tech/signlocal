# crates/websign-protocol/tests

- `common/` — shared golden fixtures and assertion helpers
- `base64_codec.rs` — Base64 codec and `Base64Bytes` (SPEC §7)
- `canonical_form.rs` — one spelling per message: repeated keys, `null`, arrays for objects, floats (SPEC §1)
- `error_codes.rs` — error codes, their wire spelling and exit codes
- `framing.rs` — length-prefixed frames (SPEC §6)
- `hello_refusal.rs` — the answer to `hello` and the version of a refusal (SPEC §5.1)
- `identifiers.rs` — `RequestId` and `FingerprintHex` (SPEC §3, §4)
- `limits_and_types.rs` — limit constants and value types
- `page_messages.rs` — page ↔ extension messages (SPEC §9)
- `parse_limits.rs` — limits checked after the body parses (SPEC §5 step 7)
- `round_trip.rs` — `parse(to_json(x)) == x` over generated messages
- `strict_envelope.rs` — envelope check order (SPEC §5 steps 1 to 5)
- `strict_fields.rs` — unknown, missing and optional fields (SPEC §5 step 6)
- `strict_values.rs` — wrong types and invalid values, never echoed (SPEC §5 step 6)
- `verification_code.rs` — the verification code vectors (SPEC §8)
- `version_negotiation.rs` — `negotiate`, `ProtocolRange` and hello ordering (SPEC §2, §5)
- `wire_format.rs` — golden JSON of every message (protocol.md §4)
