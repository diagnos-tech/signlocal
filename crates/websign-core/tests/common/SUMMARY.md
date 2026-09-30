# crates/websign-core/tests/common

- `der.rs` — The least DER needed to build test inputs by hand, plus a strict checker for what `raw_to_der` must produce.
- `mod.rs` — Helpers shared by the integration tests: fixture loading, hex, hand-made DER signatures and a blank `CertInfo`.
- `names.rs` — Distinguished names the fixtures are issued with (`gen/lib.sh`).
- `vectors.rs` — Parsers for the OpenSSL-made manifests in `tests/fixtures/vectors`.
