//! Unknown extensions, `KeyUsage` traits and `can_sign`.

use super::*;

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
