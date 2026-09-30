//! SPEC §6 and §6.5: `KeyUsage`, BasicConstraints, EKU, policies and `can_sign`.

mod common;

use common::{blank_info, info};
use websign_core::{CertInfo, KeyUsage};

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
fn key_usage_is_copy_default_and_comparable() {
    let usage = KeyUsage::default();
    let copy = usage;
    assert_eq!(usage, copy);
    assert!(ALL_BITS.iter().all(|bit| only(bit) != KeyUsage::default()));
}

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
