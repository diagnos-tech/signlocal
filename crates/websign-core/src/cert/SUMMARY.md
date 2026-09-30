# crates/websign-core/src/cert

- `der/` — the minimal DER reader and its tests
- `icp_brasil/` — ICP-Brasil profile: holder, CPF/CNPJ, level
- `tests/` — unit tests of the certificate reader over synthetic DER
- `extensions.rs` — The certificate extensions the summary reads.
- `key.rs` — Public key classification and the raw key material the verifier needs.
- `mod.rs` — Human-meaningful summary of an X.509 certificate.
- `names.rs` — Distinguished-name extraction.
- `oid.rs` — OBJECT IDENTIFIERs: dotted text, and the encoded form of every OID the summary recognizes.
- `parse.rs` — Turning certificate bytes into a [`CertInfo`].
- `qualified.rs` — EU qualified certificate statements (RFC 3739, ETSI EN 319 412-5).
- `san.rs` — SubjectAltName `otherName` entries, where ICP-Brasil keeps CPF and CNPJ.
- `strings.rs` — Text extraction from X.501 string types.
- `time.rs` — Certificate validity times as Unix seconds.
- `x509.rs` — Locates the fields of an X.509 certificate (RFC 5280 §4.1) without interpreting them.
