//! SPEC §6 (except 6.3 and 6.4): `CertInfo`, `DistinguishedName`,
//! `PublicKeyKind`, `KeyUsage`, `CertError`.

mod common;

use common::{
    Rng, UNSPECIFIED_FIXTURES, blank_info, cert, cert_names, fixture_fingerprint, info,
    validity_oracles,
};
use probe_core::{
    CertError, CertInfo, Curve, DistinguishedName, Fingerprint, IcpBrasil, KeyUsage, PublicKeyKind,
    SignatureAlgorithm,
};

fn dn(
    cn: Option<&str>,
    org: Option<&str>,
    units: &[&str],
    country: Option<&str>,
) -> DistinguishedName {
    DistinguishedName {
        common_name: cn.map(str::to_owned),
        organization: org.map(str::to_owned),
        organizational_units: units.iter().map(|u| (*u).to_owned()).collect(),
        country: country.map(str::to_owned),
    }
}

/// Subject of every ordinary fixture (`generate.sh`, `leaf_subject`).
fn leaf_dn(name: &str) -> DistinguishedName {
    dn(
        Some(&format!("Fixture {name}")),
        Some("WebeSign Test Fixtures"),
        &["Unit A", "Unit B"],
        Some("BR"),
    )
}

fn root_dn() -> DistinguishedName {
    dn(
        Some("WebeSign Test Root CA"),
        Some("WebeSign Test Authority"),
        &["Fixtures Root"],
        Some("BR"),
    )
}

fn only(bit: &str) -> KeyUsage {
    let mut usage = KeyUsage::default();
    match bit {
        "digital_signature" => usage.digital_signature = true,
        "non_repudiation" => usage.non_repudiation = true,
        "key_encipherment" => usage.key_encipherment = true,
        "data_encipherment" => usage.data_encipherment = true,
        "key_agreement" => usage.key_agreement = true,
        "key_cert_sign" => usage.key_cert_sign = true,
        "crl_sign" => usage.crl_sign = true,
        other => panic!("unknown key usage bit {other}"),
    }
    usage
}

fn usage_with(bits: &[&str]) -> KeyUsage {
    let mut usage = KeyUsage::default();
    for bit in bits {
        let one = only(bit);
        usage.digital_signature |= one.digital_signature;
        usage.non_repudiation |= one.non_repudiation;
        usage.key_encipherment |= one.key_encipherment;
        usage.data_encipherment |= one.data_encipherment;
        usage.key_agreement |= one.key_agreement;
        usage.key_cert_sign |= one.key_cert_sign;
        usage.crl_sign |= one.crl_sign;
    }
    usage
}

const ALL_BITS: [&str; 7] = [
    "digital_signature",
    "non_repudiation",
    "key_encipherment",
    "data_encipherment",
    "key_agreement",
    "key_cert_sign",
    "crl_sign",
];

// --- parsing every fixture -----------------------------------------------------------

#[test]
fn parses_every_well_formed_fixture() {
    let mut parsed = 0;
    for name in cert_names() {
        if name.starts_with("bad-") || UNSPECIFIED_FIXTURES.contains(&name.as_str()) {
            continue;
        }
        let der = cert(&name);
        let info = CertInfo::from_der(&der).unwrap_or_else(|e| panic!("{name}: {e}"));
        assert_eq!(info.fingerprint, Fingerprint::of(&der), "{name}");
        parsed += 1;
    }
    assert!(parsed > 100, "fixture set went missing");
}

#[test]
fn fingerprint_is_the_sha256_of_the_der() {
    for name in ["rsa2048", "ca", "p521", "icp-pf-a3", "qc-esign-sscd"] {
        assert_eq!(info(name).fingerprint, fixture_fingerprint(name), "{name}");
    }
}

#[test]
fn serial_and_validity_match_openssl_for_every_fixture() {
    let oracles = validity_oracles();
    assert!(oracles.len() > 100, "oracle manifest went missing");
    for oracle in oracles {
        if oracle.name.starts_with("bad-") || UNSPECIFIED_FIXTURES.contains(&oracle.name.as_str()) {
            continue;
        }
        let info = info(&oracle.name);
        assert_eq!(info.serial_hex, oracle.serial_hex, "{} serial", oracle.name);
        assert_eq!(
            info.not_before, oracle.not_before,
            "{} notBefore",
            oracle.name
        );
        assert_eq!(info.not_after, oracle.not_after, "{} notAfter", oracle.name);
    }
}

#[test]
fn cert_info_is_cloneable_and_comparable() {
    let a = info("rsa2048");
    assert_eq!(a.clone(), a);
    assert_ne!(a, info("rsa3072"));
    assert_eq!(info("rsa2048"), a, "parsing twice gives the same summary");
}

// --- names ------------------------------------------------------------------------------

#[test]
fn reads_subject_and_issuer_of_an_ordinary_certificate() {
    let info = info("rsa2048");
    assert_eq!(info.subject, leaf_dn("rsa2048"));
    assert_eq!(info.issuer, root_dn());
}

#[test]
fn a_self_signed_root_has_equal_subject_and_issuer() {
    let root = info("ca");
    assert_eq!(root.subject, root_dn());
    assert_eq!(root.issuer, root.subject);
}

#[test]
fn keeps_organizational_units_in_certificate_order() {
    assert_eq!(
        info("p256").subject.organizational_units,
        ["Unit A", "Unit B"]
    );
    assert_eq!(
        info("dn-duplicates").subject.organizational_units,
        ["A", "B", "C"]
    );
}

#[test]
fn first_occurrence_wins_for_single_valued_attributes() {
    let subject = info("dn-duplicates").subject;
    assert_eq!(subject.country.as_deref(), Some("BR"));
    assert_eq!(subject.organization.as_deref(), Some("First"));
    assert_eq!(subject.common_name.as_deref(), Some("One"));
}

#[test]
fn reads_attributes_of_a_multi_valued_rdn() {
    assert_eq!(
        info("dn-multivalue-rdn").subject,
        dn(Some("Multi"), Some("Org"), &["Unit"], Some("BR"))
    );
}

#[test]
fn decodes_string_types_as_the_spec_says() {
    let cases = [
        (
            "dn-utf8",
            dn(
                Some("JOSÉ AÇAÍ DA SILVA"),
                Some("Açougue & Cia Ltda"),
                &["Divisão São João"],
                Some("BR"),
            ),
        ),
        (
            "dn-printable",
            dn(
                Some("PRINTABLE NAME 123"),
                Some("Printable Org"),
                &["Printable Unit"],
                Some("BR"),
            ),
        ),
        (
            "dn-ia5",
            dn(
                Some("ia5.name.example.com"),
                Some("ia5.org.example"),
                &["ia5.unit"],
                Some("BR"),
            ),
        ),
        // TeletexString is Latin-1: 0xC9 is É, 0xE7 is ç, 0xE3 is ã.
        (
            "dn-teletex",
            dn(
                Some("JOSÉ TELETEX"),
                Some("Ação Ltda"),
                &["Divisão"],
                Some("BR"),
            ),
        ),
        // BMPString is UTF-16BE; Ω (U+03A9) is outside Latin-1.
        (
            "dn-bmp",
            dn(
                Some("JOSÉ Ω BMP"),
                Some("Êxito Ltda"),
                &["Divisão"],
                Some("BR"),
            ),
        ),
    ];
    for (name, expected) in cases {
        assert_eq!(info(name).subject, expected, "{name}");
    }
}

#[test]
fn ignores_attributes_of_other_string_types_and_keeps_the_rest() {
    // SPEC: NumericString and UniversalString are "other types": the attribute is ignored.
    assert_eq!(
        info("dn-numeric").subject,
        dn(None, Some("Numeric Org"), &["First", "Third"], Some("BR"))
    );
    assert_eq!(
        info("dn-universal").subject,
        dn(None, Some("Universal Org"), &["Unit"], Some("BR"))
    );
}

#[test]
fn missing_attributes_stay_none() {
    assert_eq!(info("dn-empty").subject, DistinguishedName::default());
    assert_eq!(
        info("dn-country-only").subject,
        dn(None, None, &[], Some("BR"))
    );
    assert_eq!(
        info("dn-org-only").subject,
        dn(None, Some("Only Org Name"), &[], None)
    );
}

#[test]
fn default_distinguished_name_is_empty() {
    let empty = DistinguishedName::default();
    assert_eq!(empty, dn(None, None, &[], None));
}

// --- public key ----------------------------------------------------------------------------

#[test]
fn reads_the_rsa_modulus_size_in_bits() {
    let cases = [
        ("rsa2048", 2048),
        ("rsa2048b", 2048),
        ("rsa3072", 3072),
        ("rsa4096", 4096),
        ("rsa2047", 2047),
        ("rsapss2048", 2048),
    ];
    for (name, bits) in cases {
        assert_eq!(info(name).key, PublicKeyKind::Rsa { bits }, "{name}");
    }
}

#[test]
fn reads_named_ec_curves() {
    assert_eq!(info("p256").key, PublicKeyKind::Ec { curve: Curve::P256 });
    assert_eq!(info("p256b").key, PublicKeyKind::Ec { curve: Curve::P256 });
    assert_eq!(info("p384").key, PublicKeyKind::Ec { curve: Curve::P384 });
    assert_eq!(info("p521").key, PublicKeyKind::Ec { curve: Curve::P521 });
}

#[test]
fn reports_other_algorithms_and_curves_as_unsupported_with_their_oid() {
    let cases = [
        ("ed25519", "1.3.101.112"),
        ("ed448", "1.3.101.113"),
        ("dsa1024", "1.2.840.10040.4.1"),
        // EC keys on a curve this crate does not know report the CURVE's OID.
        ("secp256k1", "1.3.132.0.10"),
        ("secp224r1", "1.3.132.0.33"),
        ("brainpoolP256r1", "1.3.36.3.3.2.8.1.1.7"),
    ];
    for (name, oid) in cases {
        assert_eq!(
            info(name).key,
            PublicKeyKind::Unsupported {
                oid: oid.to_owned()
            },
            "{name}"
        );
    }
}

#[test]
fn rsa_keys_support_both_rsa_algorithms_and_not_ecdsa() {
    let key = PublicKeyKind::Rsa { bits: 2048 };
    assert!(key.supports(SignatureAlgorithm::RsaPkcs1v15));
    assert!(key.supports(SignatureAlgorithm::RsaPss));
    assert!(!key.supports(SignatureAlgorithm::Ecdsa));
}

#[test]
fn ec_keys_support_ecdsa_only() {
    for curve in [Curve::P256, Curve::P384, Curve::P521] {
        let key = PublicKeyKind::Ec { curve };
        assert!(key.supports(SignatureAlgorithm::Ecdsa), "{curve:?}");
        assert!(!key.supports(SignatureAlgorithm::RsaPkcs1v15), "{curve:?}");
        assert!(!key.supports(SignatureAlgorithm::RsaPss), "{curve:?}");
    }
}

#[test]
fn unsupported_keys_support_nothing() {
    let key = PublicKeyKind::Unsupported {
        oid: "1.3.101.112".to_owned(),
    };
    for alg in SignatureAlgorithm::ALL {
        assert!(!key.supports(alg), "{alg}");
    }
}

// --- KeyUsage, BasicConstraints, EKU, policies -------------------------------------------------

#[test]
fn ordinary_certificates_carry_the_default_extensions() {
    let info = info("rsa2048");
    assert_eq!(
        info.key_usage,
        Some(usage_with(&["digital_signature", "non_repudiation"]))
    );
    assert_eq!(
        info.extended_key_usage,
        ["1.3.6.1.5.5.7.3.4", "1.3.6.1.5.5.7.3.2"]
    );
    assert!(info.policies.is_empty());
    assert!(!info.is_ca, "an explicit CA:FALSE means not a CA");
    assert!(info.icp_brasil.is_none());
    assert!(info.qualified.is_none());
}

#[test]
fn reads_each_key_usage_bit_on_its_own() {
    for bit in ALL_BITS {
        let name = format!("ku-{}", bit.replace('_', "-"));
        assert_eq!(info(&name).key_usage, Some(only(bit)), "{name}");
    }
}

#[test]
fn reads_all_key_usage_bits_together() {
    assert_eq!(info("ku-all").key_usage, Some(usage_with(&ALL_BITS)));
}

#[test]
fn bits_outside_the_struct_leave_it_all_false_but_present() {
    // Only decipherOnly is set: the extension exists, none of the seven bits is on.
    assert_eq!(
        info("ku-decipher-only").key_usage,
        Some(KeyUsage::default())
    );
}

#[test]
fn key_usage_is_none_without_the_extension() {
    assert_eq!(info("bare").key_usage, None);
    assert_eq!(info("v1").key_usage, None);
    assert_eq!(info("ca-pathlen0").key_usage, None);
}

#[test]
fn reads_basic_constraints() {
    assert!(info("ca").is_ca);
    assert!(info("ca-pathlen0").is_ca);
    assert!(info("ca-digital-signature").is_ca);
    assert!(!info("rsa2048").is_ca);
    assert!(!info("bare").is_ca, "no BasicConstraints at all");
    assert!(!info("v1").is_ca, "version 1 certificate");
    assert!(
        !info("ku-key-cert-sign").is_ca,
        "keyCertSign alone does not make a CA"
    );
}

#[test]
fn reads_the_root_key_usage() {
    assert_eq!(
        info("ca").key_usage,
        Some(usage_with(&["key_cert_sign", "crl_sign"]))
    );
}

#[test]
fn keeps_extended_key_usage_in_certificate_order() {
    assert_eq!(
        info("eku-multi").extended_key_usage,
        [
            "1.3.6.1.5.5.7.3.4",
            "1.3.6.1.4.1.311.10.3.12",
            "1.3.6.1.5.5.7.3.2",
            "2.5.29.37.0",
            "1.3.6.1.5.5.7.3.1",
        ]
    );
    assert_eq!(
        info("eku-server-only").extended_key_usage,
        ["1.3.6.1.5.5.7.3.1"]
    );
}

#[test]
fn extended_key_usage_is_empty_without_the_extension() {
    for name in ["bare", "v1", "ca", "ku-all", "policies-multi"] {
        assert!(info(name).extended_key_usage.is_empty(), "{name}");
    }
}

#[test]
fn keeps_policies_in_certificate_order() {
    assert_eq!(
        info("policies-multi").policies,
        ["1.2.3.4", "2.23.140.1.2.2", "1.3.6.1.4.1.99999.1.2"]
    );
}

#[test]
fn ignores_policy_qualifiers() {
    assert_eq!(
        info("policies-qualifiers").policies,
        ["1.3.6.1.4.1.99999.2.1", "1.2.3.4.5"]
    );
}

#[test]
fn policies_are_empty_without_the_extension() {
    for name in ["bare", "v1", "rsa2048", "ca"] {
        assert!(info(name).policies.is_empty(), "{name}");
    }
}

#[test]
fn skips_unknown_extensions_even_when_critical_and_garbled() {
    let info = info("unknown-extension");
    assert_eq!(info.key_usage, Some(only("digital_signature")));
    assert!(info.can_sign());
}

#[test]
fn version_1_certificate_without_extensions_has_neutral_defaults() {
    let info = info("v1");
    assert_eq!(info.subject, leaf_dn("v1"));
    assert_eq!(info.key_usage, None);
    assert!(info.extended_key_usage.is_empty());
    assert!(info.policies.is_empty());
    assert!(!info.is_ca);
    assert!(info.icp_brasil.is_none());
    assert!(info.qualified.is_none());
    assert!(info.can_sign());
}

#[test]
fn parses_a_tiny_certificate_framed_with_the_one_byte_long_length_form() {
    let der = cert("tiny");
    assert!(der.len() < 256, "fixture is meant to be under 256 bytes");
    assert_eq!(der[..2], [0x30, 0x81], "outer SEQUENCE uses 30 81 xx");
    let info = CertInfo::from_der(&der).expect("valid certificate");
    assert_eq!(info.subject, DistinguishedName::default());
    assert_eq!(info.issuer, DistinguishedName::default());
    assert_eq!(info.serial_hex, "01");
    assert_eq!(info.key, PublicKeyKind::Ec { curve: Curve::P256 });
    assert_eq!(info.key_usage, None);
    assert!(info.can_sign());
}

#[test]
fn key_usage_is_copy_default_and_comparable() {
    let usage = KeyUsage::default();
    let copy = usage;
    assert_eq!(usage, copy);
    assert!(ALL_BITS.iter().all(|bit| only(bit) != KeyUsage::default()));
}

// --- serial numbers ------------------------------------------------------------------------------

#[test]
fn drops_the_sign_byte_from_the_serial() {
    // 0x80 is encoded 00 80; twenty 0xff bytes are encoded with a leading 00.
    assert_eq!(info("serial-80").serial_hex, "80");
    assert_eq!(info("serial-ff20").serial_hex, "ff".repeat(20));
}

#[test]
fn keeps_serials_without_a_sign_byte_as_they_are() {
    assert_eq!(info("serial-01").serial_hex, "01");
    assert_eq!(info("serial-7f").serial_hex, "7f");
    assert_eq!(info("serial-1234").serial_hex, "1234");
}

#[test]
fn keeps_a_leading_zero_nibble_of_the_serial() {
    // Content octets 01 02 are "0102", not "102".
    assert_eq!(info("serial-0102").serial_hex, "0102");
}

#[test]
fn serial_is_lowercase_hex() {
    for name in ["serial-ff20", "serial-80", "p256"] {
        let serial = info(name).serial_hex;
        assert!(
            serial
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c)),
            "{serial}"
        );
        assert_eq!(serial.len() % 2, 0, "whole octets: {serial}");
    }
}

// --- validity ----------------------------------------------------------------------------------------

#[test]
fn reads_validity_as_unix_seconds() {
    let cases = [
        ("rsa2048", 1_577_836_800, 2_208_988_800), // 2020-01-01 .. 2040-01-01
        ("time-window", 1_710_498_030, 1_726_425_650), // 2024-03-15T10:20:30Z .. 2024-09-15T18:40:50Z
        ("expired", 1_262_304_000, 1_293_840_000),     // 2010 .. 2011
        ("time-y2k", 946_684_799, 946_684_800), // 1999-12-31T23:59:59Z .. 2000-01-01T00:00:00Z
    ];
    for (name, not_before, not_after) in cases {
        let info = info(name);
        assert_eq!(
            (info.not_before, info.not_after),
            (not_before, not_after),
            "{name}"
        );
    }
}

#[test]
fn reads_utc_time_before_1970_as_a_negative_number() {
    // UTCTime "600101000000Z" is 1960 (years 50..99 mean 19xx).
    let info = info("time-pre-1970");
    assert_eq!(info.not_before, -315_619_200);
    assert_eq!(
        info.not_after, 2_524_607_999,
        "2049-12-31T23:59:59Z is still UTCTime"
    );
}

#[test]
fn reads_generalized_time_from_2050_on() {
    let info = info("time-generalized");
    assert_eq!(info.not_before, 2_524_608_000);
    assert_eq!(info.not_after, 2_556_144_000);
}

#[test]
fn reads_the_no_expiry_date_of_year_9999() {
    assert_eq!(info("time-no-expiry").not_after, 253_402_300_799);
}

#[test]
fn validity_window_is_inclusive_at_both_ends() {
    let info = info("time-window");
    let (from, to) = (info.not_before, info.not_after);
    assert!(!info.is_valid_at(from - 1));
    assert!(info.is_valid_at(from));
    assert!(info.is_valid_at(from + 1));
    assert!(info.is_valid_at(to - 1));
    assert!(info.is_valid_at(to));
    assert!(!info.is_valid_at(to + 1));
}

#[test]
fn expired_fixture_is_not_valid_today_but_was_valid_then() {
    let info = info("expired");
    assert!(!info.is_valid_at(1_800_000_000));
    assert!(info.is_valid_at(1_270_000_000));
    assert!(!info.is_valid_at(0));
}

#[test]
fn validity_check_on_a_hand_built_summary() {
    let info = CertInfo {
        not_before: 1_000,
        not_after: 2_000,
        ..blank_info()
    };
    let cases = [
        (i64::MIN, false),
        (-1, false),
        (0, false),
        (999, false),
        (1_000, true),
        (1_500, true),
        (2_000, true),
        (2_001, false),
        (i64::MAX, false),
    ];
    for (t, expected) in cases {
        assert_eq!(info.is_valid_at(t), expected, "t = {t}");
    }
}

#[test]
fn validity_check_survives_the_extremes_of_i64() {
    let always = CertInfo {
        not_before: i64::MIN,
        not_after: i64::MAX,
        ..blank_info()
    };
    assert!(always.is_valid_at(i64::MIN));
    assert!(always.is_valid_at(0));
    assert!(always.is_valid_at(i64::MAX));

    let instant = CertInfo {
        not_before: 42,
        not_after: 42,
        ..blank_info()
    };
    assert!(instant.is_valid_at(42));
    assert!(!instant.is_valid_at(41));
    assert!(!instant.is_valid_at(43));

    let inverted = CertInfo {
        not_before: 2_000,
        not_after: 1_000,
        ..blank_info()
    };
    for t in [999, 1_000, 1_500, 2_000, 2_001] {
        assert!(!inverted.is_valid_at(t), "t = {t}");
    }
}

// --- can_sign ----------------------------------------------------------------------------------------

#[test]
fn can_sign_follows_ca_flag_and_key_usage() {
    let sign = only("digital_signature");
    let commit = only("non_repudiation");
    let neither = usage_with(&[
        "key_encipherment",
        "data_encipherment",
        "key_agreement",
        "key_cert_sign",
        "crl_sign",
    ]);
    let cases: Vec<(bool, Option<KeyUsage>, bool, &str)> = vec![
        (false, None, true, "no KeyUsage extension"),
        (true, None, false, "CA without KeyUsage"),
        (
            false,
            Some(KeyUsage::default()),
            false,
            "extension with no bit on",
        ),
        (false, Some(sign), true, "digitalSignature"),
        (false, Some(commit), true, "nonRepudiation"),
        (
            false,
            Some(usage_with(&["digital_signature", "non_repudiation"])),
            true,
            "both signing bits",
        ),
        (false, Some(usage_with(&ALL_BITS)), true, "every bit"),
        (false, Some(neither), false, "every non-signing bit"),
        (true, Some(sign), false, "CA with digitalSignature"),
        (true, Some(commit), false, "CA with nonRepudiation"),
        (
            true,
            Some(usage_with(&ALL_BITS)),
            false,
            "CA with every bit",
        ),
    ];
    for (is_ca, key_usage, expected, label) in cases {
        let info = CertInfo {
            is_ca,
            key_usage,
            ..blank_info()
        };
        assert_eq!(info.can_sign(), expected, "{label}");
    }
}

#[test]
fn can_sign_is_false_for_each_non_signing_bit_alone() {
    for bit in [
        "key_encipherment",
        "data_encipherment",
        "key_agreement",
        "key_cert_sign",
        "crl_sign",
    ] {
        let info = CertInfo {
            key_usage: Some(only(bit)),
            ..blank_info()
        };
        assert!(!info.can_sign(), "{bit}");
    }
}

#[test]
fn can_sign_ignores_extended_key_usage() {
    for eku in [
        vec![],
        vec!["1.3.6.1.5.5.7.3.1".to_owned()],
        vec!["2.5.29.37.0".to_owned()],
    ] {
        let info = CertInfo {
            key_usage: Some(only("digital_signature")),
            extended_key_usage: eku.clone(),
            ..blank_info()
        };
        assert!(info.can_sign(), "{eku:?}");
        let info = CertInfo {
            key_usage: Some(only("key_agreement")),
            extended_key_usage: eku.clone(),
            ..blank_info()
        };
        assert!(!info.can_sign(), "{eku:?}");
    }
}

#[test]
fn can_sign_on_the_fixtures() {
    let expected = [
        ("rsa2048", true),
        ("p256", true),
        ("bare", true),
        ("v1", true),
        ("ku-digital-signature", true),
        ("ku-non-repudiation", true),
        ("ku-all", true),
        ("eku-server-only", true),
        ("ku-key-encipherment", false),
        ("ku-data-encipherment", false),
        ("ku-key-agreement", false),
        ("ku-key-cert-sign", false),
        ("ku-crl-sign", false),
        ("ku-decipher-only", false),
        ("ca", false),
        ("ca-pathlen0", false),
        ("ca-digital-signature", false),
    ];
    for (name, can_sign) in expected {
        assert_eq!(info(name).can_sign(), can_sign, "{name}");
    }
}

#[test]
fn key_encipherment_certificate_cannot_sign() {
    // The scenario in the brief: an encryption-only certificate on a token.
    let info = info("ku-key-encipherment");
    assert_eq!(info.key_usage, Some(only("key_encipherment")));
    assert!(!info.can_sign());
}

// --- display_name ----------------------------------------------------------------------------------------

fn named(cn: Option<&str>, org: Option<&str>) -> DistinguishedName {
    dn(cn, org, &["Some Unit"], Some("BR"))
}

#[test]
fn display_name_prefers_the_icp_brasil_holder() {
    let info = CertInfo {
        subject: named(Some("CN Name:123"), Some("Org")),
        icp_brasil: Some(IcpBrasil {
            holder_name: Some("HOLDER".to_owned()),
            ..IcpBrasil::default()
        }),
        ..blank_info()
    };
    assert_eq!(info.display_name(), "HOLDER");
}

#[test]
fn display_name_falls_back_to_the_common_name() {
    let icp_without_holder = Some(IcpBrasil::default());
    for icp_brasil in [None, icp_without_holder] {
        let info = CertInfo {
            subject: named(Some("Common Name"), Some("Org")),
            icp_brasil,
            ..blank_info()
        };
        assert_eq!(info.display_name(), "Common Name");
    }
}

#[test]
fn display_name_falls_back_to_the_organization() {
    let info = CertInfo {
        subject: named(None, Some("Only Org")),
        ..blank_info()
    };
    assert_eq!(info.display_name(), "Only Org");
}

#[test]
fn display_name_last_resort_is_the_first_16_hex_digits_of_the_fingerprint() {
    let info = CertInfo {
        subject: named(None, None),
        ..blank_info()
    };
    assert_eq!(info.display_name(), "abababababababab");

    let bytes: [u8; 32] = std::array::from_fn(|i| i as u8 * 7 + 1);
    let fingerprint = Fingerprint::from_bytes(bytes);
    let info = CertInfo {
        fingerprint,
        subject: DistinguishedName::default(),
        ..blank_info()
    };
    assert_eq!(info.display_name(), fingerprint.to_hex()[..16]);
    assert_eq!(info.display_name().len(), 16);
}

#[test]
fn display_name_does_not_use_units_or_country() {
    let info = CertInfo {
        subject: dn(None, None, &["Unit"], Some("BR")),
        ..blank_info()
    };
    assert_eq!(info.display_name(), "abababababababab");
}

#[test]
fn display_name_on_the_fixtures() {
    assert_eq!(info("rsa2048").display_name(), "Fixture rsa2048");
    assert_eq!(info("icp-pf-a3").display_name(), "ANA BEATRIZ SOUZA");
    assert_eq!(info("dn-org-only").display_name(), "Only Org Name");
    // Not ICP-Brasil, so the ":digits" suffix stays.
    assert_eq!(info("non-icp-upn").display_name(), "PLAIN NAME:12345678901");
    // No CN in the numeric-CN fixture (ignored): the organization is next.
    assert_eq!(info("dn-numeric").display_name(), "Numeric Org");
    for name in ["dn-empty", "dn-country-only"] {
        let info = info(name);
        assert_eq!(
            info.display_name(),
            fixture_fingerprint(name).to_hex()[..16],
            "{name}"
        );
    }
}

// --- errors -----------------------------------------------------------------------------------------

fn assert_malformed(input: &[u8], label: &str) {
    match CertInfo::from_der(input) {
        Err(CertError::Malformed(_)) => {}
        other => panic!("{label}: expected Malformed, got {other:?}"),
    }
}

#[test]
fn rejects_bytes_that_are_not_a_certificate() {
    let pem = b"-----BEGIN CERTIFICATE-----\nMIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEA\n-----END CERTIFICATE-----\n";
    let cases: Vec<(&str, Vec<u8>)> = vec![
        ("empty", vec![]),
        ("one byte", vec![0x30]),
        ("empty SEQUENCE", vec![0x30, 0x00]),
        (
            "SEQUENCE with one INTEGER",
            vec![0x30, 0x03, 0x02, 0x01, 0x00],
        ),
        (
            "SEQUENCE with three INTEGERs",
            vec![0x30, 0x09, 2, 1, 0, 2, 1, 0, 2, 1, 0],
        ),
        ("zeros", vec![0; 64]),
        ("ones", vec![0xFF; 64]),
        ("ASCII text", b"hello world".to_vec()),
        ("PEM instead of DER", pem.to_vec()),
        ("INTEGER", vec![0x02, 0x01, 0x05]),
        (
            "length beyond the data",
            vec![0x30, 0x82, 0xFF, 0xFF, 0x30, 0x00],
        ),
        ("indefinite length", vec![0x30, 0x80, 0x00, 0x00]),
    ];
    for (label, bytes) in cases {
        assert_malformed(&bytes, label);
    }
}

#[test]
fn rejects_every_truncation_of_a_certificate() {
    for name in ["p256", "icp-pf-a3"] {
        let der = cert(name);
        for len in 0..der.len() {
            assert_malformed(&der[..len], &format!("{name} cut at {len}"));
        }
    }
}

#[test]
fn rejects_bytes_after_the_certificate() {
    let der = cert("rsa2048");
    let tails: [&[u8]; 5] = [&[0], &[0, 0, 0, 0], &[0x30, 0x00], b"\n", &der];
    for tail in tails {
        let mut padded = der.clone();
        padded.extend_from_slice(tail);
        assert_malformed(&padded, &format!("{} trailing bytes", tail.len()));
    }
}

#[test]
fn rejects_a_certificate_rewritten_with_indefinite_length() {
    // A well-formed certificate re-framed as BER (30 80 ... 00 00) is not DER.
    let der = cert("p256");
    let header = if der[1] == 0x82 { 4 } else { 3 };
    let mut ber = vec![0x30, 0x80];
    ber.extend_from_slice(&der[header..]);
    ber.extend_from_slice(&[0, 0]);
    assert_malformed(&ber, "indefinite length");
}

#[test]
fn rejects_known_extensions_that_are_malformed() {
    for name in [
        "bad-ext-key-usage",
        "bad-ext-extended-key-usage",
        "bad-ext-policies",
        "bad-ext-basic-constraints",
        "bad-ext-san",
        "bad-ext-qc-statements",
        "bad-qc-statement-element",
    ] {
        assert_malformed(&cert(name), name);
    }
}

#[test]
fn malformed_error_message_is_prefixed_and_carries_detail() {
    let err = CertInfo::from_der(&[0x30]).unwrap_err();
    assert!(
        err.to_string().starts_with("malformed certificate: "),
        "{err}"
    );
    let CertError::Malformed(detail) = &err;
    assert_eq!(err.to_string(), format!("malformed certificate: {detail}"));
}

#[test]
fn cert_error_is_a_cloneable_comparable_std_error() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    assert_error::<CertError>();
    let err = CertError::Malformed("x".to_owned());
    assert_eq!(err.clone(), err);
    assert_ne!(err, CertError::Malformed("y".to_owned()));
}

// --- never panics ----------------------------------------------------------------------------------------

#[test]
fn survives_a_flipped_byte_anywhere_in_a_certificate() {
    for name in ["icp-pf-a3", "qc-esign-sscd", "dn-bmp", "eku-multi"] {
        let der = cert(name);
        for index in 0..der.len() {
            for mask in [0xFF, 0x01, 0x80] {
                let mut broken = der.clone();
                broken[index] ^= mask;
                // Either outcome is fine; panicking is not.
                let _ = CertInfo::from_der(&broken);
            }
        }
    }
}

#[test]
fn survives_random_garbage() {
    let mut rng = Rng::new(0xC0FFEE);
    for _ in 0..2000 {
        let len = rng.below(400) as usize;
        let mut bytes = rng.bytes(len);
        if let Some(first) = bytes.first_mut()
            && rng.below(2) == 0
        {
            *first = 0x30;
        }
        let _ = CertInfo::from_der(&bytes);
    }
}

#[test]
fn survives_a_valid_certificate_with_its_body_replaced_by_garbage() {
    let der = cert("p256");
    let mut rng = Rng::new(7);
    for _ in 0..300 {
        let mut broken = der.clone();
        let start = 4 + rng.below((der.len() - 8) as u64) as usize;
        let end = (start + 1 + rng.below(20) as usize).min(der.len());
        broken[start..end].copy_from_slice(&rng.bytes(end - start));
        let _ = CertInfo::from_der(&broken);
    }
}
