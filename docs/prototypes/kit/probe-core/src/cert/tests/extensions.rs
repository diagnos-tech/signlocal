use super::*;

#[test]
fn key_usage_bits_and_can_sign() {
    let with_usage = |bits: &[usize]| parse(&TestCert::new().extension(key_usage(bits)));

    let info = with_usage(&[0, 1, 2, 3, 4, 5, 6]);
    assert_eq!(
        info.key_usage,
        Some(KeyUsage {
            digital_signature: true,
            non_repudiation: true,
            key_encipherment: true,
            data_encipherment: true,
            key_agreement: true,
            key_cert_sign: true,
            crl_sign: true,
        })
    );

    assert!(with_usage(&[0]).can_sign());
    assert!(with_usage(&[1]).can_sign());
    assert!(with_usage(&[0, 2]).can_sign());
    assert!(!with_usage(&[2]).can_sign());
    assert!(!with_usage(&[4]).can_sign());
    assert!(!with_usage(&[5, 6]).can_sign());
}

#[test]
fn missing_key_usage_does_not_block_signing() {
    let info = parse(&TestCert::new());
    assert_eq!(info.key_usage, None);
    assert!(info.can_sign());
}

#[test]
fn key_usage_bit_positions_beyond_ours_are_ignored() {
    // decipherOnly (bit 8) needs a second byte.
    let info = parse(&TestCert::new().extension(key_usage(&[8])));
    assert_eq!(info.key_usage, Some(KeyUsage::default()));
    assert!(!info.can_sign());
}

#[test]
fn a_ca_cannot_sign_even_with_signature_usage() {
    let ca = TestCert::new()
        .extension(basic_constraints(true))
        .extension(key_usage(&[0, 5]));
    let info = parse(&ca);
    assert!(info.is_ca);
    assert!(!info.can_sign());

    let leaf = parse(&TestCert::new().extension(basic_constraints(false)));
    assert!(!leaf.is_ca);
    assert!(leaf.can_sign());
}

#[test]
fn extended_key_usage_and_policies_keep_certificate_order() {
    let info = parse(
        &TestCert::new()
            .extension(extended_key_usage(&[
                "1.3.6.1.5.5.7.3.4",
                "1.3.6.1.5.5.7.3.2",
                "1.3.6.1.4.1.311.10.3.12",
            ]))
            .extension(policies(&["2.16.76.1.2.3.1", "2.5.29.32.0", "1.2.3.4"])),
    );
    assert_eq!(
        info.extended_key_usage,
        [
            "1.3.6.1.5.5.7.3.4",
            "1.3.6.1.5.5.7.3.2",
            "1.3.6.1.4.1.311.10.3.12"
        ]
    );
    assert_eq!(info.policies, ["2.16.76.1.2.3.1", "2.5.29.32.0", "1.2.3.4"]);
}

#[test]
fn unknown_extensions_are_ignored_even_when_their_content_is_junk() {
    let info = parse(&TestCert::new().extension(raw_extension("1.2.3.4.5", &[0xde, 0xad])));
    assert_eq!(info.key_usage, None);
}

#[test]
fn malformed_known_extensions_reject_the_certificate() {
    let junk = [0x04, 0x01, 0x00];
    for extension_oid in [
        "2.5.29.15",
        "2.5.29.17",
        "2.5.29.19",
        "2.5.29.32",
        "2.5.29.37",
        "1.3.6.1.5.5.7.1.3",
    ] {
        let der = TestCert::new()
            .extension(raw_extension(extension_oid, &junk))
            .build();
        assert!(
            matches!(CertInfo::from_der(&der), Err(CertError::Malformed(_))),
            "{extension_oid}"
        );
    }
}

#[test]
fn first_copy_of_a_repeated_extension_wins() {
    let info = parse(
        &TestCert::new()
            .extension(key_usage(&[0]))
            .extension(key_usage(&[2])),
    );
    assert!(
        info.key_usage
            .is_some_and(|u| u.digital_signature && !u.key_encipherment)
    );
}
