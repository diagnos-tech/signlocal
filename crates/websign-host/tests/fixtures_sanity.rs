//! The test fixtures themselves: they must verify, or every other test would
//! fail for a reason that has nothing to do with the host.

mod common;

use common::Cert;
use websign_core::{HashAlgorithm, SignatureAlgorithm, verify};

#[test]
fn every_fixture_signature_verifies_against_its_certificate() {
    let cases = [
        (Cert::p256(), vec![SignatureAlgorithm::Ecdsa]),
        (Cert::person(), vec![SignatureAlgorithm::Ecdsa]),
        (
            Cert::rsa(),
            vec![SignatureAlgorithm::RsaPkcs1v15, SignatureAlgorithm::RsaPss],
        ),
        (
            Cert::rsa_b(),
            vec![SignatureAlgorithm::RsaPkcs1v15, SignatureAlgorithm::RsaPss],
        ),
    ];
    for (cert, algorithms) in cases {
        for algorithm in algorithms {
            for hash in HashAlgorithm::ALL {
                let result = verify(
                    &cert.der,
                    hash,
                    algorithm,
                    &common::certs::digest(hash),
                    &cert.signature(hash, algorithm),
                );
                assert_eq!(result, Ok(()), "{} {algorithm:?} {hash:?}", cert.name);
            }
        }
    }
}

#[test]
fn the_person_fixture_carries_personal_data_the_log_test_looks_for() {
    let person = Cert::person();
    let name = person.info.display_name();
    assert!(name.to_uppercase().contains("ANA BEATRIZ"), "{name}");
    assert_ne!(person.hex(), Cert::p256().hex());
}
