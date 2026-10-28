//! `Curve` properties and `from_oid`.

use super::*;

#[test]
fn reports_field_length_in_bytes() {
    let expected = [32, 48, 66, 32, 48, 64];
    for (curve, len) in ALL.into_iter().zip(expected) {
        assert_eq!(curve.field_len(), len, "{curve:?}");
    }
}

#[test]
fn reports_raw_signature_length_as_twice_the_field() {
    for curve in ALL {
        assert_eq!(curve.signature_len(), 2 * curve.field_len(), "{curve:?}");
    }
    assert_eq!(BrainpoolP256r1.signature_len(), 64);
    assert_eq!(BrainpoolP384r1.signature_len(), 96);
    assert_eq!(BrainpoolP512r1.signature_len(), 128);
}

#[test]
fn reports_curve_names() {
    assert_eq!(BrainpoolP256r1.name(), "brainpoolP256r1");
    assert_eq!(BrainpoolP384r1.name(), "brainpoolP384r1");
    assert_eq!(BrainpoolP512r1.name(), "brainpoolP512r1");
    let names: HashSet<&str> = ALL.iter().map(|c| c.name()).collect();
    assert_eq!(names.len(), 6);
}

#[test]
fn only_brainpool_p512r1_lacks_a_verifier() {
    for curve in ALL {
        assert_eq!(curve.has_verifier(), curve != BrainpoolP512r1, "{curve:?}");
    }
}

#[test]
fn maps_the_brainpool_r1_oids() {
    assert_eq!(
        Curve::from_oid("1.3.36.3.3.2.8.1.1.7"),
        Some(BrainpoolP256r1)
    );
    assert_eq!(
        Curve::from_oid("1.3.36.3.3.2.8.1.1.11"),
        Some(BrainpoolP384r1)
    );
    assert_eq!(
        Curve::from_oid("1.3.36.3.3.2.8.1.1.13"),
        Some(BrainpoolP512r1)
    );
}

#[test]
fn keeps_the_nist_oids_working() {
    assert_eq!(Curve::from_oid("1.2.840.10045.3.1.7"), Some(P256));
    assert_eq!(Curve::from_oid("1.3.132.0.34"), Some(P384));
    assert_eq!(Curve::from_oid("1.3.132.0.35"), Some(P521));
}

#[test]
fn leaves_the_twisted_brainpool_oids_unknown() {
    for (_, oid) in BRAINPOOL_TWISTED {
        assert_eq!(Curve::from_oid(oid), None, "{oid}");
    }
}

#[test]
fn rejects_brainpool_look_alike_oids() {
    let unknown = [
        "1.3.36.3.3.2.8.1.1.5",  // brainpoolP224r1
        "1.3.36.3.3.2.8.1.1.6",  // brainpoolP224t1
        "1.3.36.3.3.2.8.1.1.9",  // P320r1
        "1.3.36.3.3.2.8.1.1.10", // P320t1
        "1.3.36.3.3.2.8.1.1.15",
        "1.3.36.3.3.2.8.1.1.70",
        "1.3.36.3.3.2.8.1.1.7.1",
        "1.3.36.3.3.2.8.1.1.7 ",
        " 1.3.36.3.3.2.8.1.1.11",
        "1.3.36.3.3.2.8.1.1",
        "1.3.36.3.3.2.8.1.1.113",
        "1.3.36.3.3.2.8.1.1.07",
        "brainpoolP256r1",
        "brainpoolP384r1",
    ];
    for oid in unknown {
        assert_eq!(Curve::from_oid(oid), None, "{oid:?}");
    }
}

#[test]
fn the_six_curve_oids_are_distinct() {
    let oids = [
        "1.2.840.10045.3.1.7",
        "1.3.132.0.34",
        "1.3.132.0.35",
        "1.3.36.3.3.2.8.1.1.7",
        "1.3.36.3.3.2.8.1.1.11",
        "1.3.36.3.3.2.8.1.1.13",
    ];
    let curves: HashSet<Curve> = oids
        .iter()
        .map(|oid| Curve::from_oid(oid).expect("known curve"))
        .collect();
    assert_eq!(curves.len(), 6);
}
