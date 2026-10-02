//! SPEC §2: `SignatureAlgorithm`, `UnknownAlgorithmError`.

use std::collections::HashSet;

use probe_core::{SignatureAlgorithm, UnknownAlgorithmError};

use SignatureAlgorithm::{Ecdsa, RsaPkcs1v15, RsaPss};

#[test]
fn lists_all_algorithms_in_declaration_order() {
    assert_eq!(SignatureAlgorithm::ALL, [Ecdsa, RsaPkcs1v15, RsaPss]);
}

#[test]
fn reports_webcrypto_names() {
    assert_eq!(Ecdsa.name(), "ECDSA");
    assert_eq!(RsaPkcs1v15.name(), "RSASSA-PKCS1-v1_5");
    assert_eq!(RsaPss.name(), "RSASSA-PSS");
}

#[test]
fn says_which_algorithms_need_an_rsa_key() {
    assert!(!Ecdsa.is_rsa());
    assert!(RsaPkcs1v15.is_rsa());
    assert!(RsaPss.is_rsa());
}

#[test]
fn displays_the_webcrypto_name() {
    for alg in SignatureAlgorithm::ALL {
        assert_eq!(alg.to_string(), alg.name());
        assert_eq!(format!("{alg}"), alg.name());
    }
}

#[test]
fn is_copy_eq_and_hashable() {
    let alg = RsaPss;
    let copy = alg;
    assert_eq!(alg, copy);
    assert_ne!(Ecdsa, RsaPss);
    let set: HashSet<SignatureAlgorithm> = SignatureAlgorithm::ALL.into_iter().collect();
    assert_eq!(set.len(), 3);
}

#[test]
fn parses_the_exact_names() {
    assert_eq!("ECDSA".parse(), Ok(Ecdsa));
    assert_eq!("RSASSA-PKCS1-v1_5".parse(), Ok(RsaPkcs1v15));
    assert_eq!("RSASSA-PSS".parse(), Ok(RsaPss));
}

#[test]
fn parses_names_ignoring_case() {
    let cases = [
        ("ecdsa", Ecdsa),
        ("Ecdsa", Ecdsa),
        ("eCdSa", Ecdsa),
        ("rsassa-pkcs1-v1_5", RsaPkcs1v15),
        ("RSASSA-PKCS1-V1_5", RsaPkcs1v15),
        ("Rsassa-Pkcs1-V1_5", RsaPkcs1v15),
        ("rsassa-pss", RsaPss),
        ("Rsassa-Pss", RsaPss),
    ];
    for (text, expected) in cases {
        assert_eq!(text.parse::<SignatureAlgorithm>(), Ok(expected), "{text:?}");
    }
}

#[test]
fn rejects_anything_that_is_not_one_of_the_three_names() {
    let rejected = [
        "",
        " ",
        " ECDSA",
        "ECDSA ",
        "ECDSA\n",
        "ECDSA-SHA256",
        "ES256",
        "ES384",
        "P-256",
        "EC",
        "RSA",
        "RSASSA",
        "RSA-PSS",
        "RSAPSS",
        "RSASSA_PSS",
        "RSASSA-PSS ",
        "RSASSA-PKCS1",
        "RSASSA-PKCS1-v1.5",
        "RSASSA-PKCS1-v1-5",
        "RSASSA-PKCS1_v1_5",
        "RSASSA-PKCS1-v1_5 ",
        "PKCS1",
        "PKCS1v15",
        "PSS",
        "RS256",
        "PS256",
        "Ed25519",
        "SHA-256",
        "ＥＣＤＳＡ",
        "ECDSA\0",
    ];
    for text in rejected {
        assert_eq!(
            text.parse::<SignatureAlgorithm>(),
            Err(UnknownAlgorithmError {
                name: text.to_owned()
            }),
            "{text:?}"
        );
    }
}

#[test]
fn keeps_the_original_casing_in_the_error() {
    let err = "Rsa-Pss".parse::<SignatureAlgorithm>().unwrap_err();
    assert_eq!(err.name, "Rsa-Pss");
}

#[test]
fn unknown_algorithm_error_displays_the_offending_name() {
    let err = "RSA".parse::<SignatureAlgorithm>().unwrap_err();
    assert_eq!(err.to_string(), "unknown algorithm: RSA");
    let err = "".parse::<SignatureAlgorithm>().unwrap_err();
    assert_eq!(err.to_string(), "unknown algorithm: ");
}

#[test]
fn unknown_algorithm_error_is_a_cloneable_comparable_std_error() {
    fn assert_error<E: std::error::Error + Clone + PartialEq + Send + Sync + 'static>() {}
    assert_error::<UnknownAlgorithmError>();
    let err = UnknownAlgorithmError {
        name: "x".to_owned(),
    };
    assert_eq!(err.clone(), err);
}

#[test]
fn round_trips_name_and_display_through_from_str() {
    for alg in SignatureAlgorithm::ALL {
        assert_eq!(alg.name().parse::<SignatureAlgorithm>(), Ok(alg));
        assert_eq!(alg.to_string().parse::<SignatureAlgorithm>(), Ok(alg));
        assert_eq!(
            alg.name().to_uppercase().parse::<SignatureAlgorithm>(),
            Ok(alg)
        );
        assert_eq!(
            alg.name().to_lowercase().parse::<SignatureAlgorithm>(),
            Ok(alg)
        );
    }
}

#[test]
fn does_not_accept_hash_names() {
    for text in ["SHA-256", "SHA-384", "SHA-512", "SHA256"] {
        assert!(text.parse::<SignatureAlgorithm>().is_err(), "{text}");
    }
}
