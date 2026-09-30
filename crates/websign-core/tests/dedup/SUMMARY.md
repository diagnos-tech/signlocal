# crates/websign-core/tests/dedup

One test binary; `main.rs` holds the shared helpers.

- `main.rs` — SPEC §8: `dedup_by_fingerprint`, `SourceKind`, `Deduped`.
- `trivial_inputs.rs` — trivial inputs
- `primary.rs` — choosing the primary
- `order.rs` — order and grouping
- `generic.rs` — generic over the item type
- `types.rs` — types
