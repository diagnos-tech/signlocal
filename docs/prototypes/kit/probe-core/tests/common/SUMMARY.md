# probe-core/tests/common

Helpers shared by the integration tests.

- `der.rs` — minimal DER builder and a strict checker for `raw_to_der` output
- `mod.rs` — fixture loading, hex, hand-made DER signatures, a blank `CertInfo`
- `names.rs` — the distinguished names the fixtures are issued with
- `vectors.rs` — parsers for the OpenSSL-made manifests in `fixtures/vectors`
