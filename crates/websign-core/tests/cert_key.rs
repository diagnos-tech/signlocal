//! SPEC §6.2: `PublicKeyKind`.

mod common;

use common::{BRAINPOOL_R1_KEYS, BRAINPOOL_TWISTED, info};
use websign_core::{Curve, PublicKeyKind, SignatureAlgorithm};

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

#[test]
fn reads_brainpool_r1_curves_as_named_ec_keys() {
    // SPEC §4.1: these summarize as `Ec`, not `Unsupported`.
    for (name, curve) in BRAINPOOL_R1_KEYS {
        assert_eq!(info(name).key, PublicKeyKind::Ec { curve }, "{name}");
    }
    assert_eq!(
        info("brainpoolP256r1").key,
        PublicKeyKind::Ec {
            curve: Curve::BrainpoolP256r1
        }
    );
}

#[test]
fn reports_twisted_brainpool_curves_as_unsupported_with_the_curve_oid() {
    for (name, oid) in BRAINPOOL_TWISTED {
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
fn brainpool_ec_keys_support_ecdsa_only() {
    for (name, _) in BRAINPOOL_R1_KEYS {
        let key = info(name).key;
        assert!(key.supports(SignatureAlgorithm::Ecdsa), "{name}");
        assert!(!key.supports(SignatureAlgorithm::RsaPkcs1v15), "{name}");
        assert!(!key.supports(SignatureAlgorithm::RsaPss), "{name}");
    }
}

#[test]
fn brainpool_certificates_are_ordinary_signing_certificates() {
    for (name, _) in BRAINPOOL_R1_KEYS {
        let info = info(name);
        assert!(info.can_sign(), "{name}");
        assert!(info.is_valid_at(1_800_000_000), "{name}");
    }
}
