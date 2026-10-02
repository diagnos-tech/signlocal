# probe-core/src/cert/tests

Unit tests for `CertInfo` over hand-built DER, grouped by what they exercise.

- `basics.rs` — basic parsing and fields
- `extensions.rs` — extension handling
- `keys.rs` — public key classification
- `mod.rs` — shared setup and overview
- `names.rs` — name encodings and extraction
- `profiles.rs` — ICP-Brasil and eIDAS profiles
- `tolerance.rs` — encodings that break strict DER but occur in the field and must not hide a certificate
