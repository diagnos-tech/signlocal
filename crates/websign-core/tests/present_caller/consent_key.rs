//! `consent_key` families and stability.

use super::*;

#[test]
fn an_authenticode_key_has_the_subject_and_the_lowercase_file_name() {
    let c = caller(
        "/Apps/Acme/AcmeSigner.EXE",
        None,
        authenticode("Acme Corp Ltda"),
    );
    assert_eq!(
        consent_key(&c),
        "app:authenticode:Acme Corp Ltda:acmesigner.exe"
    );
}

#[test]
fn the_authenticode_subject_keeps_its_case() {
    let c = caller("/x/a.exe", None, authenticode("ACME Corp"));
    assert_eq!(consent_key(&c), "app:authenticode:ACME Corp:a.exe");
}

#[test]
fn an_apple_key_has_team_id_and_identifier() {
    let c = caller(
        "/Applications/Acme.app/Contents/MacOS/Acme",
        Some("Acme"),
        apple("ABCDE12345", "com.acme.signer"),
    );
    assert_eq!(consent_key(&c), "app:apple:ABCDE12345:com.acme.signer");
}

#[test]
fn an_apple_key_ignores_the_path_and_the_product_name() {
    let a = caller(
        "/Applications/A.app/Contents/MacOS/A",
        Some("A"),
        apple("T1", "com.a"),
    );
    let b = caller("/Users/x/Downloads/B", Some("B"), apple("T1", "com.a"));
    assert_eq!(consent_key(&a), consent_key(&b));
}

#[test]
fn an_unsigned_key_is_the_path_as_given() {
    let c = caller("/home/ana/Tools/Acme-CLI", Some("Acme"), None);
    assert_eq!(consent_key(&c), "path:/home/ana/Tools/Acme-CLI");
}

#[test]
fn an_unsigned_key_is_not_canonicalized() {
    for path in [
        "./rel/../App",
        "relative/app",
        "/a//b/./c",
        "/opt/UPPER/app",
    ] {
        let c = caller(path, None, None);
        assert_eq!(consent_key(&c), format!("path:{path}"), "{path}");
    }
}

#[test]
fn a_signed_key_survives_the_binary_moving() {
    let before = caller("/Apps/Acme/acme.exe", None, authenticode("Acme Corp"));
    let after = caller(
        "/Users/ana/AppData/Acme/acme.exe",
        None,
        authenticode("Acme Corp"),
    );
    assert_eq!(consent_key(&before), consent_key(&after));
}

#[test]
fn the_file_name_in_an_authenticode_key_is_case_insensitive() {
    let a = caller("/x/Acme.EXE", None, authenticode("Acme Corp"));
    let b = caller("/y/acme.exe", None, authenticode("Acme Corp"));
    assert_eq!(consent_key(&a), consent_key(&b));
}

#[test]
fn different_signers_or_programs_get_different_keys() {
    let base = caller("/x/acme.exe", None, authenticode("Acme Corp"));
    let other_signer = caller("/x/acme.exe", None, authenticode("Evil Corp"));
    let other_file = caller("/x/other.exe", None, authenticode("Acme Corp"));
    let unsigned = caller("/x/acme.exe", None, None);
    let keys = [
        consent_key(&base),
        consent_key(&other_signer),
        consent_key(&other_file),
        consent_key(&unsigned),
    ];
    for (i, a) in keys.iter().enumerate() {
        for b in &keys[i + 1..] {
            assert_ne!(a, b);
        }
    }
}

#[test]
fn an_unsigned_program_cannot_forge_a_signed_key() {
    // The key prefixes keep the three families apart.
    let forged = caller("app:apple:ABCDE12345:com.acme.signer", None, None);
    let real = caller("/x/y", None, apple("ABCDE12345", "com.acme.signer"));
    assert_eq!(
        consent_key(&forged),
        "path:app:apple:ABCDE12345:com.acme.signer"
    );
    assert_ne!(consent_key(&forged), consent_key(&real));
}

#[test]
fn never_panics_on_odd_callers() {
    let long = "x".repeat(10_000);
    for path in ["", "/", "..", "a", "\u{0}", "/é/ü.exe", long.as_str()] {
        for product in [None, Some(""), Some("é"), Some(long.as_str())] {
            for signer in [
                None,
                authenticode(""),
                authenticode("é"),
                apple("", ""),
                apple("T", "é"),
            ] {
                let c = caller(path, product, signer);
                let _ = caller_label(&c);
                let _ = consent_key(&c);
            }
        }
    }
}
