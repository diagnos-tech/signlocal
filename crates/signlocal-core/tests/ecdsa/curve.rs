//! Curve.

use super::*;

#[test]
fn reports_field_length_in_bytes() {
    assert_eq!(P256.field_len(), 32);
    assert_eq!(P384.field_len(), 48);
    assert_eq!(P521.field_len(), 66);
}

#[test]
fn reports_raw_signature_length_as_twice_the_field() {
    assert_eq!(P256.signature_len(), 64);
    assert_eq!(P384.signature_len(), 96);
    assert_eq!(P521.signature_len(), 132);
}

#[test]
fn reports_nist_names() {
    assert_eq!(P256.name(), "P-256");
    assert_eq!(P384.name(), "P-384");
    assert_eq!(P521.name(), "P-521");
}

#[test]
fn curve_is_copy_eq_and_hashable() {
    let curve = P256;
    let copy = curve;
    assert_eq!(curve, copy);
    assert_ne!(P256, P384);
    assert_eq!(ALL_CURVES.into_iter().collect::<HashSet<_>>().len(), 3);
}

#[test]
fn maps_named_curve_oids() {
    assert_eq!(Curve::from_oid("1.2.840.10045.3.1.7"), Some(P256));
    assert_eq!(Curve::from_oid("1.3.132.0.34"), Some(P384));
    assert_eq!(Curve::from_oid("1.3.132.0.35"), Some(P521));
}

#[test]
fn rejects_other_oids_and_look_alikes() {
    let unknown = [
        "",
        " ",
        "1.2.840.10045.3.1.7 ",
        " 1.3.132.0.34",
        "1.3.132.0.33", // secp224r1
        "1.3.132.0.10", // secp256k1
        "1.3.132.0.36", // sect283k1 neighbour
        "1.3.132.0.3",
        "1.3.132.0.340",
        "1.3.132.0.34.1",
        "1.3.132.0.35.0",
        "1.2.840.10045.3.1.1", // prime192v1
        "1.2.840.10045.3.1.70",
        "1.2.840.10045.2.1", // id-ecPublicKey, not a curve
        "1.3.101.112",       // Ed25519
        "prime256v1",
        "secp384r1",
        "P-256",
        "P-521",
        "2.16.840.1.101.3.4.2.1",
        "not an oid",
    ];
    for oid in unknown {
        assert_eq!(Curve::from_oid(oid), None, "{oid:?}");
    }
}

#[test]
fn formats_encoding_errors() {
    assert_eq!(Malformed.to_string(), "malformed ECDSA signature");
    assert_eq!(
        IntegerTooLarge.to_string(),
        "ECDSA integer does not fit the curve"
    );
    assert_eq!(
        WrongLength {
            expected: 64,
            actual: 63
        }
        .to_string(),
        "raw ECDSA signature must be 64 bytes, got 63"
    );
}

#[test]
fn encoding_error_is_a_cloneable_comparable_std_error() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    assert_error::<EcdsaEncodingError>();
    assert_ne!(Malformed, IntegerTooLarge);
}
