//! Construction and accessors.

use super::*;

#[test]
fn computes_sha256_of_the_input() {
    assert_eq!(Fingerprint::of(b"abc").to_hex(), ABC_HEX);
    assert_eq!(Fingerprint::of(b"").to_hex(), EMPTY_HEX);
}

#[test]
fn hashes_every_certificate_fixture_like_openssl() {
    let names = cert_names();
    assert!(names.len() > 100, "fixture set went missing");
    for name in names {
        assert_eq!(
            Fingerprint::of(&cert(&name)),
            fixture_fingerprint(&name),
            "{name}"
        );
    }
}

#[test]
fn different_inputs_have_different_fingerprints() {
    assert_ne!(Fingerprint::of(b"a"), Fingerprint::of(b"b"));
    assert_ne!(Fingerprint::of(b""), Fingerprint::of(&[0]));
    assert_eq!(Fingerprint::of(b"a"), Fingerprint::of(b"a"));
}

#[test]
fn from_bytes_and_as_bytes_are_inverse() {
    for start in [0u8, 1, 0x80, 0xFF] {
        let bytes = bytes_32(start);
        assert_eq!(Fingerprint::from_bytes(bytes).as_bytes(), &bytes);
    }
}

#[test]
fn of_stores_the_raw_digest_bytes() {
    assert_eq!(
        Fingerprint::of(b"abc").as_bytes()[..],
        hex_decode(ABC_HEX)[..]
    );
}
