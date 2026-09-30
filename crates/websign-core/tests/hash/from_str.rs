//! FromStr.

use super::*;

#[test]
fn parses_every_accepted_spelling_ignoring_case() {
    let cases = [
        ("SHA-256", Sha256),
        ("SHA256", Sha256),
        ("sha-256", Sha256),
        ("sha256", Sha256),
        ("Sha-256", Sha256),
        ("sHa256", Sha256),
        ("SHA-384", Sha384),
        ("SHA384", Sha384),
        ("sha-384", Sha384),
        ("sha384", Sha384),
        ("SHA-512", Sha512),
        ("SHA512", Sha512),
        ("sha-512", Sha512),
        ("sha512", Sha512),
        ("ShA-512", Sha512),
    ];
    for (text, expected) in cases {
        assert_eq!(text.parse::<HashAlgorithm>(), Ok(expected), "{text:?}");
    }
}

#[test]
fn rejects_everything_else_with_the_original_string() {
    let rejected = [
        "",
        " ",
        " SHA-256",
        "SHA-256 ",
        " SHA-256 ",
        "SHA-256\n",
        "\tSHA-256",
        "SHA-1",
        "SHA1",
        "SHA-224",
        "SHA224",
        "SHA-512/256",
        "SHA512/256",
        "SHA3-256",
        "SHA3-512",
        "SHA_256",
        "SHA 256",
        "SHA--256",
        "SHA-2560",
        "SHA-25",
        "SHA-",
        "SHA",
        "256",
        "-256",
        "MD5",
        "sha256sum",
        "SHA-256\0",
        "SHA\u{2011}256",
        "ＳＨＡ-256",
        "SHA-２５６",
        "ECDSA",
        "RSASSA-PSS",
    ];
    for text in rejected {
        assert_eq!(
            text.parse::<HashAlgorithm>(),
            Err(UnknownAlgorithmError {
                name: text.to_owned()
            }),
            "{text:?}"
        );
    }
}

#[test]
fn case_folding_is_ascii_only() {
    // Case-insensitive means ASCII case-insensitive.
    // U+017F (long s) upper-cases to `S` under full Unicode rules and must
    // not sneak in as an `s`.
    assert!("ſha-256".parse::<HashAlgorithm>().is_err());
    assert!("ſha512".parse::<HashAlgorithm>().is_err());
}

#[test]
fn unknown_hash_error_displays_the_offending_name() {
    let err = "SHA-1".parse::<HashAlgorithm>().unwrap_err();
    assert_eq!(err.to_string(), "unknown algorithm: SHA-1");
    let err = " SHA-256".parse::<HashAlgorithm>().unwrap_err();
    assert_eq!(err.to_string(), "unknown algorithm:  SHA-256");
    let err = "".parse::<HashAlgorithm>().unwrap_err();
    assert_eq!(err.to_string(), "unknown algorithm: ");
}

#[test]
fn round_trips_name_and_display_through_from_str() {
    for hash in HashAlgorithm::ALL {
        assert_eq!(hash.name().parse::<HashAlgorithm>(), Ok(hash));
        assert_eq!(hash.to_string().parse::<HashAlgorithm>(), Ok(hash));
        assert_eq!(
            hash.name().to_lowercase().parse::<HashAlgorithm>(),
            Ok(hash)
        );
    }
}

#[test]
fn does_not_accept_signature_algorithm_names() {
    for text in ["ECDSA", "RSASSA-PKCS1-v1_5", "RSASSA-PSS"] {
        assert!(text.parse::<HashAlgorithm>().is_err(), "{text}");
    }
}
