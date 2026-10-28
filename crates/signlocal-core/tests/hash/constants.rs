//! Constants.

use super::*;

#[test]
fn lists_all_algorithms_from_shortest_digest_to_longest() {
    assert_eq!(HashAlgorithm::ALL, [Sha256, Sha384, Sha512]);
}

#[test]
fn reports_digest_length_in_bytes() {
    assert_eq!(Sha256.digest_len(), 32);
    assert_eq!(Sha384.digest_len(), 48);
    assert_eq!(Sha512.digest_len(), 64);
}

#[test]
fn reports_webcrypto_names() {
    assert_eq!(Sha256.name(), "SHA-256");
    assert_eq!(Sha384.name(), "SHA-384");
    assert_eq!(Sha512.name(), "SHA-512");
}

#[test]
fn displays_the_webcrypto_name() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(hash.to_string(), hash.name());
        assert_eq!(format!("{hash}"), hash.name());
    }
}

#[test]
fn is_copy_eq_and_hashable() {
    let copy = Sha256;
    let again = copy;
    assert_eq!(copy, again);
    assert_ne!(Sha256, Sha384);
    let set: HashSet<HashAlgorithm> = HashAlgorithm::ALL.into_iter().collect();
    assert_eq!(set.len(), 3);
}
