//! `KeyUsage`, BasicConstraints, EKU and policies as read.

use super::*;

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
