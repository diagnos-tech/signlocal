//! Invalid signatures (part 1 of 3).

use super::*;

#[test]
fn rejects_a_digest_that_was_not_signed() {
    for vector in valid_vectors() {
        let digest = fixture_digest(vector.hash);
        // ECDSA uses only the leftmost field-size bytes of a longer digest
        // (FIPS 186-5), so flips beyond them are legitimately invisible.
        let significant = match vector.key.as_str() {
            "p256" | "p256b" => digest.len().min(32),
            "p384" => digest.len().min(48),
            _ => digest.len(),
        };
        for index in [0, significant / 2, significant - 1] {
            let mut tampered = digest.clone();
            tampered[index] ^= 0x01;
            let result = verify(
                &cert(&vector.key),
                vector.hash,
                algorithm_of(&vector.algorithm),
                &tampered,
                &vector.signature,
            );
            assert_eq!(
                result,
                Err(VerifyError::InvalidSignature),
                "{} {} byte {index}",
                vector.key,
                vector.algorithm
            );
        }
    }
}

#[test]
fn rejects_a_signature_with_one_flipped_bit() {
    for vector in valid_vectors() {
        let len = vector.signature.len();
        for index in [0, len / 2, len - 1] {
            for mask in [0x01, 0x80] {
                let mut tampered = vector.signature.clone();
                tampered[index] ^= mask;
                let result = verify(
                    &cert(&vector.key),
                    vector.hash,
                    algorithm_of(&vector.algorithm),
                    &fixture_digest(vector.hash),
                    &tampered,
                );
                assert_eq!(
                    result,
                    Err(VerifyError::InvalidSignature),
                    "{} {} {} byte {index} ^ {mask:#x}",
                    vector.key,
                    vector.algorithm,
                    vector.hash
                );
            }
        }
    }
}

#[test]
fn rejects_a_signature_made_over_another_hash_of_the_same_message() {
    for vector in valid_vectors() {
        for other in HashAlgorithm::ALL.into_iter().filter(|h| *h != vector.hash) {
            let result = verify(
                &cert(&vector.key),
                other,
                algorithm_of(&vector.algorithm),
                &fixture_digest(other),
                &vector.signature,
            );
            assert_eq!(
                result,
                Err(VerifyError::InvalidSignature),
                "{} {} {} as {other}",
                vector.key,
                vector.algorithm,
                vector.hash
            );
        }
    }
}

#[test]
fn rejects_a_signature_of_the_wrong_length() {
    for vector in valid_vectors() {
        let sig = &vector.signature;
        let mut with_zero_prefix = vec![0];
        with_zero_prefix.extend_from_slice(sig);
        let mut with_zero_suffix = sig.clone();
        with_zero_suffix.push(0);
        let cases: Vec<(&str, Vec<u8>)> = vec![
            ("empty", vec![]),
            ("one byte", vec![sig[0]]),
            ("one byte short (tail)", sig[..sig.len() - 1].to_vec()),
            ("one byte short (head)", sig[1..].to_vec()),
            ("one byte long (suffix)", with_zero_suffix),
            ("one byte long (prefix)", with_zero_prefix),
            ("doubled", [sig.as_slice(), sig.as_slice()].concat()),
        ];
        for (label, bad) in cases {
            let result = verify(
                &cert(&vector.key),
                vector.hash,
                algorithm_of(&vector.algorithm),
                &fixture_digest(vector.hash),
                &bad,
            );
            assert_eq!(
                result,
                Err(VerifyError::InvalidSignature),
                "{} {}: {label}",
                vector.key,
                vector.algorithm
            );
        }
    }
}
