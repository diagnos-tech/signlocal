# crates/websign-core/src/cert/tests

- `basics.rs` — serial, times, versions and structure errors
- `extensions.rs` — KeyUsage, EKU, policies, BasicConstraints, duplicates
- `keys.rs` — public key kinds and curves
- `mod.rs` — Unit tests for `CertInfo` over hand-built DER (`testkit`), grouped by what they exercise.
- `names.rs` — distinguished-name string types and duplicates
- `profiles.rs` — ICP-Brasil and qcStatements on synthetic certificates
- `tolerance.rs` — Encodings that break strict DER or RFC 5280 but occur in certificates in the field.
