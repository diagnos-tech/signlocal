# probe-core/tests

Integration tests written from `SPEC.md`, one file per section or subsection, over OpenSSL-made fixtures.

- `algorithm.rs` — spec §2: signature algorithm names
- `cert.rs` — spec §6: certificate parsing and errors
- `cert_icp_brasil.rs` — spec §6.3: ICP-Brasil detection, level, holder, CPF, CNPJ
- `cert_key.rs` — spec §6.2: public key kinds
- `cert_names.rs` — spec §6.1 and §6.5: distinguished names and display name
- `cert_qualified.rs` — spec §6.4: eIDAS qcStatements
- `cert_usage.rs` — spec §6: key usage, basic constraints, EKU, policies, `can_sign`
- `cert_validity.rs` — spec §6: serial, validity, `is_valid_at`
- `common/` — shared test helpers
- `dedup.rs` — spec §8: deduplication
- `ecdsa.rs` — spec §4: curves and DER/raw conversion
- `fingerprint.rs` — spec §5: certificate fingerprint
- `fixtures/` — OpenSSL-made certificates, reference vectors, and their generator
- `hash.rs` — spec §1: hash algorithms and digest length
- `pkcs1.rs` — spec §3: `DigestInfo`
- `verify.rs` — spec §7: signature verification
