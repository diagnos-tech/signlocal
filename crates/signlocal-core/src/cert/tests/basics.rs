use super::*;

#[test]
fn defaults_describe_a_plain_certificate() {
    let cert = TestCert::new();
    let der = cert.build();
    let info = CertInfo::from_der(&der).unwrap();

    assert_eq!(info.fingerprint, Fingerprint::of(&der));
    assert_eq!(info.subject.common_name.as_deref(), Some("Test Subject"));
    assert_eq!(info.issuer.common_name.as_deref(), Some("Test CA"));
    assert_eq!(info.serial_hex, "01");
    assert_eq!(info.not_before, JAN_1_2024);
    assert_eq!(info.not_after, JAN_1_2025);
    assert_eq!(info.key, PublicKeyKind::Rsa { bits: 2048 });
    assert_eq!(info.key_usage, None);
    assert!(info.extended_key_usage.is_empty());
    assert!(info.policies.is_empty());
    assert!(!info.is_ca);
    assert_eq!(info.icp_brasil, None);
    assert_eq!(info.qualified, None);
}

#[test]
fn serial_numbers_are_hex_without_the_sign_byte() {
    let serial = |content: &[u8]| parse(&TestCert::new().serial(content)).serial_hex;
    assert_eq!(serial(&[0x00, 0x80]), "80");
    assert_eq!(serial(&[0x12, 0x34]), "1234");
    assert_eq!(serial(&[0x7f]), "7f");
    assert_eq!(serial(&[0x00, 0xff, 0x01]), "ff01");
    let twenty = [&[0x00][..], &[0xfe; 20]].concat();
    assert_eq!(serial(&twenty), "fe".repeat(20));
}

#[test]
fn validity_reads_utc_and_generalized_time() {
    let info = parse(&TestCert::new().validity("700101000000Z", "20500101000000Z"));
    assert_eq!(info.not_before, 0);
    assert_eq!(info.not_after, 2_524_608_000);

    let info = parse(&TestCert::new().validity("491231235959Z", "20991231235959Z"));
    assert_eq!(info.not_before, 2_524_607_999);
    assert_eq!(info.not_after, 4_102_444_799);
}

#[test]
fn validity_bounds_are_inclusive() {
    let info = parse(&TestCert::new());
    assert!(!info.is_valid_at(JAN_1_2024 - 1));
    assert!(info.is_valid_at(JAN_1_2024));
    assert!(info.is_valid_at((JAN_1_2024 + JAN_1_2025) / 2));
    assert!(info.is_valid_at(JAN_1_2025));
    assert!(!info.is_valid_at(JAN_1_2025 + 1));
}

#[test]
fn structurally_invalid_certificates_are_malformed() {
    let good = TestCert::new().build();
    assert!(CertInfo::from_der(&good).is_ok());

    let mut trailing = good.clone();
    trailing.push(0x00);
    let truncated = good[..good.len() - 1].to_vec();
    let mut wrong_tag = good.clone();
    wrong_tag[0] = 0x31;

    for (why, bytes) in [
        ("empty", Vec::new()),
        ("one byte", vec![0x30]),
        ("empty sequence", vec![0x30, 0x00]),
        ("trailing byte", trailing),
        ("truncated", truncated),
        ("wrong outer tag", wrong_tag),
        ("text", b"-----BEGIN CERTIFICATE-----".to_vec()),
    ] {
        assert!(
            matches!(CertInfo::from_der(&bytes), Err(CertError::Malformed(_))),
            "{why}"
        );
    }
}

#[test]
fn malformed_error_message_names_the_problem() {
    let err = CertInfo::from_der(&[0x30, 0x00]).unwrap_err();
    assert!(err.to_string().starts_with("malformed certificate: "));
}

#[test]
fn corrupting_a_valid_certificate_never_panics() {
    let der = icp_cert()
        .extension(qc_statements(&[("0.4.0.1862.1.1", None)]))
        .extension(extended_key_usage(&["1.3.6.1.5.5.7.3.2"]))
        .build();
    for cut in 0..der.len() {
        let _ = CertInfo::from_der(&der[..cut]);
    }
    for index in 0..der.len() {
        for flip in [0x01, 0x80, 0xff] {
            let mut mutated = der.clone();
            mutated[index] ^= flip;
            let _ = CertInfo::from_der(&mutated);
        }
    }
}
