//! Invalid signatures (part 2 of 3).

use super::*;

#[test]
fn rejects_ecdsa_in_der_form_because_the_sdk_promises_raw() {
    for vector in ecdsa_vectors() {
        let key = match vector.curve {
            Curve::P256 => "p256",
            Curve::P384 => "p384",
            Curve::P521 => "p521",
            other => panic!("no fixture key for {other:?}"),
        };
        assert_eq!(
            verify_fixture(key, vector.hash, Ecdsa, &vector.der),
            Err(VerifyError::InvalidSignature),
            "{key} {}",
            vector.shape
        );
    }
}

#[test]
fn rejects_degenerate_signature_blocks_of_the_right_size() {
    let sizes = [
        ("rsa2048", RsaPkcs1v15, 256),
        ("rsa2048", RsaPss, 256),
        ("rsa3072", RsaPkcs1v15, 384),
        ("p256", Ecdsa, 64),
        ("p384", Ecdsa, 96),
        ("p521", Ecdsa, 132),
    ];
    for (key, algorithm, len) in sizes {
        for (label, fill) in [("all zero", 0x00), ("all ff", 0xFF), ("all 01", 0x01)] {
            let result = verify_fixture(key, Sha256, algorithm, &vec![fill; len]);
            assert_eq!(
                result,
                Err(VerifyError::InvalidSignature),
                "{key} {algorithm}: {label}"
            );
        }
    }
}

/// `a + b` for big-endian numbers of equal length; the sum must fit.
fn add(a: &[u8], b: &[u8]) -> Vec<u8> {
    let mut sum = vec![0; a.len()];
    let mut carry = 0;
    for i in (0..a.len()).rev() {
        let digit = u16::from(a[i]) + u16::from(b[i]) + carry;
        sum[i] = digit as u8;
        carry = digit >> 8;
    }
    assert_eq!(carry, 0, "sum overflows the block");
    sum
}

#[test]
fn rejects_rsa_signatures_that_are_not_below_the_modulus() {
    // RFC 8017 §5.2.2: s must be below n. s + n has the same residue as a
    // valid s, and a verifier that reduces instead of rejecting (the `rsa`
    // crate's PSS verifier does) would accept it as a second signature. A
    // 2047-bit modulus leaves room for s + n in the 256-byte block.
    let modulus = left_pad(&rsa_modulus(&cert("rsa2047")), 256);
    for (kind, algorithm) in [("pkcs1", RsaPkcs1v15), ("pss", RsaPss)] {
        for hash in HashAlgorithm::ALL {
            let valid = signature("rsa2047", kind, hash);
            for (label, bad) in [("n", modulus.clone()), ("s + n", add(&valid, &modulus))] {
                assert_eq!(
                    verify_fixture("rsa2047", hash, algorithm, &bad),
                    Err(VerifyError::InvalidSignature),
                    "{kind} {hash}: {label}"
                );
            }
        }
    }
}

#[test]
fn rejects_ecdsa_with_a_zero_or_out_of_range_component() {
    let good = signature("p256", "ecdsa", Sha256);
    let (r, s) = good.split_at(32);
    let order = hex_decode("FFFFFFFF00000000FFFFFFFFFFFFFFFFBCE6FAADA7179E84F3B9CAC2FC632551");
    let one = [vec![0; 31], vec![1]].concat();
    let cases = [
        ("r = 0", [vec![0; 32], s.to_vec()].concat()),
        ("s = 0", [r.to_vec(), vec![0; 32]].concat()),
        ("r = n", [order.clone(), s.to_vec()].concat()),
        ("s = n", [r.to_vec(), order.clone()].concat()),
        ("r = 2^256-1", [vec![0xFF; 32], s.to_vec()].concat()),
        ("s = 2^256-1", [r.to_vec(), vec![0xFF; 32]].concat()),
        ("r = s = 1", [one.clone(), one].concat()),
        ("swapped r and s", [s, r].concat()),
    ];
    for (label, sig) in cases {
        assert_eq!(
            verify_fixture("p256", Sha256, Ecdsa, &sig),
            Err(VerifyError::InvalidSignature),
            "{label}"
        );
    }
}

#[test]
fn rejects_a_signature_from_another_key_of_the_same_kind() {
    let cases = [
        ("rsa2048", "rsa2048b", RsaPkcs1v15, "pkcs1"),
        ("rsa2048", "rsa2048b", RsaPss, "pss"),
        ("p256", "p256b", Ecdsa, "ecdsa"),
    ];
    for (cert_key, signer, algorithm, kind) in cases {
        for hash in HashAlgorithm::ALL {
            let sig = signature(signer, kind, hash);
            assert_eq!(
                verify_fixture(signer, hash, algorithm, &sig),
                Ok(()),
                "control: {signer} {kind} {hash}"
            );
            assert_eq!(
                verify_fixture(cert_key, hash, algorithm, &sig),
                Err(VerifyError::InvalidSignature),
                "{signer}'s {kind} against {cert_key} with {hash}"
            );
        }
    }
}
