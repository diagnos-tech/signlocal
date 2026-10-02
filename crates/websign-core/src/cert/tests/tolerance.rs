//! Encodings that break strict DER or RFC 5280 but occur in certificates in
//! the field. None of them changes the summary, so none may hide a
//! certificate from the list.

use super::*;

#[test]
fn reads_times_before_1970_and_generalized_time_before_2050() {
    let info = parse(&TestCert::new().validity("500101000000Z", "19991231235959Z"));
    assert_eq!(info.not_before, -631_152_000);
    assert_eq!(info.not_after, 946_684_799);
}

#[test]
fn rejects_time_forms_rfc_5280_forbids() {
    for (not_before, not_after) in [
        ("2401010000Z", "250101000000Z"),
        ("240101000000Z", "20250101000000.5Z"),
        ("240230000000Z", "250101000000Z"),
    ] {
        let der = TestCert::new().validity(not_before, not_after).build();
        assert!(
            matches!(CertInfo::from_der(&der), Err(CertError::Malformed(_))),
            "{not_before} {not_after}"
        );
    }
}

#[test]
fn reads_serials_of_any_length_and_sign() {
    let serial = |content: &[u8]| parse(&TestCert::new().serial(content)).serial_hex;
    let long = [0x5a; 32];
    assert_eq!(serial(&long), "5a".repeat(32), "longer than 20 octets");
    assert_eq!(serial(&[0xff, 0x01]), "ff01", "negative");
    assert_eq!(serial(&[0x00]), "00", "zero");
    assert_eq!(serial(&[0x00, 0x00, 0x05]), "05", "redundant zeros");
    let empty = TestCert::new().serial(&[]).build();
    assert!(CertInfo::from_der(&empty).is_err());
}

#[test]
fn accepts_an_explicit_default_version_and_a_missing_one() {
    for version in [None, Some(0), Some(1), Some(2)] {
        let info = parse(&TestCert::new().version(version).extension(key_usage(&[0])));
        assert!(info.key_usage.is_some(), "{version:?}");
    }
}

#[test]
fn accepts_explicit_false_and_ber_true_booleans() {
    let key_usage_value = tlv(0x03, &[0x07, 0x80]);
    for critical in [Some(0x00), Some(0x01), Some(0xff), None] {
        let ext = extension_with_criticality("2.5.29.15", critical, &key_usage_value);
        let info = parse(&TestCert::new().extension(ext));
        assert!(info.key_usage.is_some_and(|u| u.digital_signature));
    }
    let ber_ca = extension_with_criticality("2.5.29.19", None, &sequence(&[tlv(0x01, &[0x01])]));
    assert!(parse(&TestCert::new().extension(ber_ca)).is_ca);
    let explicit_leaf =
        extension_with_criticality("2.5.29.19", None, &sequence(&[tlv(0x01, &[0x00])]));
    assert!(!parse(&TestCert::new().extension(explicit_leaf)).is_ca);
}

#[test]
fn reads_unsorted_multi_valued_rdns() {
    // DER sorts SET OF by encoding; CN (55 04 03) would come before OU (55 04 0b).
    let ou = sequence(&[oid(OU), tlv(UTF8, b"Unit")]);
    let cn = sequence(&[oid(CN), tlv(UTF8, b"Multi")]);
    let rdn = tlv(0x31, &[ou, cn].concat());
    let dn = parse(&TestCert::new().subject(&[rdn])).subject;
    assert_eq!(dn.common_name.as_deref(), Some("Multi"));
    assert_eq!(dn.organizational_units, ["Unit"]);
}

#[test]
fn reads_non_minimal_lengths() {
    // A UTF8String with `82 00 04` and a SET with `81 0f`, where the short
    // form would do.
    let value = [&[UTF8, 0x82, 0x00, 0x04][..], b"Long"].concat();
    let attribute = sequence(&[oid(CN), value]);
    let rdn = [&[0x31, 0x81, attribute.len() as u8][..], &attribute].concat();
    let info = parse(&TestCert::new().subject_der(sequence(&[rdn])));
    assert_eq!(info.subject.common_name.as_deref(), Some("Long"));
}

#[test]
fn reads_rsa_moduli_without_a_sign_byte_or_with_redundant_zeros() {
    let mut negative = vec![0xff; 256];
    negative[255] = 0xfd;
    let padded = [&[0x00, 0x00, 0x00][..], &negative].concat();
    for modulus in [negative, padded] {
        let info = parse(&TestCert::new().spki(spki_rsa_integers(&modulus, &[0x01, 0x00, 0x01])));
        assert_eq!(info.key, PublicKeyKind::Rsa { bits: 2048 });
    }
}

#[test]
fn reads_rsa_pss_keys_with_parameters_as_rsa() {
    // id-RSASSA-PSS with an (empty) RSASSA-PSS-params SEQUENCE.
    let key = sequence(&[tlv(0x02, &[0x00, 0xc1, 0x02, 0x03]), tlv(0x02, &[0x03])]);
    let algorithm = sequence(&[oid("1.2.840.113549.1.1.10"), sequence(&[])]);
    let spki = sequence(&[algorithm, bit_string(&key)]);
    assert_eq!(
        parse(&TestCert::new().spki(spki)).key,
        PublicKeyKind::Rsa { bits: 24 }
    );
}

#[test]
fn reads_ec_keys_with_explicit_parameters_as_unsupported_ec() {
    let explicit = sequence(&[tlv(0x02, &[1]), sequence(&[])]);
    let algorithm = sequence(&[oid("1.2.840.10045.2.1"), explicit]);
    let spki = sequence(&[algorithm, bit_string(&[0x04, 0x01, 0x02])]);
    assert_eq!(
        parse(&TestCert::new().spki(spki)).key,
        PublicKeyKind::Unsupported {
            oid: "1.2.840.10045.2.1".into()
        }
    );
}

#[test]
fn a_key_usage_with_no_bit_set_is_present_and_forbids_signing() {
    let empty = extension_with_criticality("2.5.29.15", Some(0xff), &tlv(0x03, &[0x00]));
    let info = parse(&TestCert::new().extension(empty));
    assert_eq!(info.key_usage, Some(KeyUsage::default()));
    assert!(!info.can_sign());
}

#[test]
fn skips_unique_ids() {
    let info = parse(
        &TestCert::new()
            .unique_id(tlv(0x81, &[0x00, 0x01]))
            .unique_id(tlv(0x82, &[0x00, 0x02]))
            .extension(key_usage(&[1])),
    );
    assert!(info.key_usage.is_some_and(|u| u.non_repudiation));
}

#[test]
fn rejects_unexpected_fields_in_the_tbs_certificate() {
    let der = TestCert::new().unique_id(tlv(0x02, &[0x05])).build();
    assert!(CertInfo::from_der(&der).is_err());
}

#[test]
fn an_oid_too_large_to_print_is_ignored_unless_it_must_be_printed() {
    // 1.2.(2^133): the last arc needs twenty 7-bit groups and exceeds u128.
    let huge = [&[0x06, 21, 0x2a, 0x81][..], &[0x80; 18], &[0x00]].concat();
    let unknown_attribute = tlv(0x31, &sequence(&[huge.clone(), tlv(UTF8, b"x")]));
    let unknown_extension = sequence(&[huge.clone(), tlv(0x04, &[0x05, 0x00])]);
    let info = parse(
        &TestCert::new()
            .subject(&[unknown_attribute, rdn(CN, UTF8, b"Kept")])
            .extension(unknown_extension),
    );
    assert_eq!(info.subject.common_name.as_deref(), Some("Kept"));

    let policy = sequence(&[sequence(&[huge])]);
    let der = TestCert::new()
        .extension(raw_extension("2.5.29.32", &policy))
        .build();
    assert!(matches!(
        CertInfo::from_der(&der),
        Err(CertError::Malformed(_))
    ));
}

#[test]
fn prints_policy_arcs_beyond_u32() {
    let info = parse(&TestCert::new().extension(policies(&["2.16.76.1.2.4294967296.1"])));
    assert_eq!(info.policies, ["2.16.76.1.2.4294967296.1"]);
    let icp = info.icp_brasil.expect("still ICP-Brasil");
    assert_eq!(icp.level, None, "IcpLevel::Other cannot hold the arc");
}

#[test]
fn the_first_readable_value_of_a_single_valued_attribute_wins() {
    let dn = subject_of(&[
        rdn(CN, 0x12, b"12345"),
        rdn(CN, UTF8, &[0xff]),
        rdn(CN, UTF8, b"Readable"),
        rdn(CN, UTF8, b"Later"),
    ]);
    assert_eq!(dn.common_name.as_deref(), Some("Readable"));
}

#[test]
fn reads_uuid_oids_under_2_25_anywhere() {
    // A 128-bit arc: `const-oid` (behind `x509-cert`) stops at u32 and would
    // have rejected the whole certificate over any one of these.
    let uuid = "2.25.329800735698586629295641978511506172918";
    let info = parse(
        &TestCert::new()
            .subject(&[rdn(uuid, UTF8, b"x"), rdn(CN, UTF8, b"Kept")])
            .extension(raw_extension(uuid, &[0x05, 0x00]))
            .extension(policies(&[uuid]))
            .extension(extended_key_usage(&[uuid])),
    );
    assert_eq!(info.subject.common_name.as_deref(), Some("Kept"));
    assert_eq!(info.policies, [uuid]);
    assert_eq!(info.extended_key_usage, [uuid]);
}
