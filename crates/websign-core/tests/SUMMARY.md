# crates/websign-core/tests

- `common/` — shared helpers of the integration tests
- `fixtures/` — OpenSSL-generated certificates and vectors
- `algorithm.rs` — SPEC §2: `SignatureAlgorithm`, `UnknownAlgorithmError`.
- `cert.rs` — SPEC §6 and §6.0: parsing `CertInfo` from DER, and `CertError`.
- `cert_icp_brasil.rs` — SPEC §6.3: ICP-Brasil detection, level, holder, CPF and CNPJ.
- `cert_key.rs` — SPEC §6.2: `PublicKeyKind`.
- `cert_names.rs` — SPEC §6.1 and `display_name` (§6.5): `DistinguishedName`.
- `cert_qualified.rs` — SPEC §6.4: eIDAS qcStatements (`Qualified`, `QcType`).
- `cert_usage.rs` — SPEC §6 and §6.5: `KeyUsage`, BasicConstraints, EKU, policies and `can_sign`.
- `cert_validity.rs` — SPEC §6: `serial_hex`, `not_before`, `not_after` and `is_valid_at` (§6.5).
- `dedup.rs` — SPEC §8: `dedup_by_fingerprint`, `SourceKind`, `Deduped`.
- `ecdsa.rs` — SPEC §4: `Curve`, `der_to_raw`, `raw_to_der`.
- `fingerprint.rs` — SPEC §5: `Fingerprint`, `ParseFingerprintError`.
- `hash.rs` — SPEC §1: `HashAlgorithm`, `DigestLengthError`.
- `pkcs1.rs` — SPEC §3: `pkcs1::digest_info`.
- `verify.rs` — SPEC §7: `verify`, `VerifyError`.
