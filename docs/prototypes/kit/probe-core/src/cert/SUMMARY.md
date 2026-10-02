# probe-core/src/cert

Human-meaningful summary of an X.509 certificate, built on a tolerant DER reader.

- `der/` — the tolerant DER reader
- `extensions.rs` — the extensions the summary reads
- `icp_brasil/` — ICP-Brasil level and holder identifiers
- `key.rs` — public key classification and raw key material for the verifier
- `mod.rs` — `CertInfo` and the summary's public types
- `names.rs` — distinguished-name extraction
- `oid.rs` — OBJECT IDENTIFIERS: dotted text and encoded form of every recognized OID
- `parse.rs` — turns certificate bytes into a `CertInfo`
- `qualified.rs` — EU qualified certificate statements (RFC 3739, ETSI EN 319 412-5)
- `san.rs` — SubjectAltName `otherName` entries, where ICP-Brasil keeps CPF and CNPJ
- `strings.rs` — text extraction from X.501 string types
- `tests/` — unit tests over hand-built DER
- `time.rs` — validity times as Unix seconds, including instants before 1970
- `x509.rs` — locates the fields of a certificate without interpreting them
