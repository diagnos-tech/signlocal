//! SPEC §12: `present::caller`.
//!
//! Paths use `/` only, which every supported OS treats as a separator.

use std::path::PathBuf;

use websign_core::present::caller::{
    CallerLabel, CodeSigner, DesktopCaller, caller_label, consent_key,
};

fn caller(path: &str, product: Option<&str>, signer: Option<CodeSigner>) -> DesktopCaller {
    DesktopCaller {
        executable: PathBuf::from(path),
        product_name: product.map(str::to_owned),
        signer,
    }
}

fn authenticode(subject: &str) -> Option<CodeSigner> {
    Some(CodeSigner::Authenticode {
        subject: subject.to_owned(),
    })
}

fn apple(team_id: &str, identifier: &str) -> Option<CodeSigner> {
    Some(CodeSigner::Apple {
        team_id: team_id.to_owned(),
        identifier: identifier.to_owned(),
    })
}

fn label(name: &str, detail: &str, verified: bool) -> CallerLabel {
    CallerLabel {
        name: name.to_owned(),
        detail: detail.to_owned(),
        verified,
    }
}

// --- caller_label: name ------------------------------------------------------------------------

#[test]
fn the_name_is_the_product_name_when_there_is_one() {
    let c = caller("/opt/acme/bin/acme-cli", Some("Acme Signer"), None);
    assert_eq!(caller_label(&c).name, "Acme Signer");
}

#[test]
fn the_name_is_trimmed() {
    let c = caller("/opt/acme/app", Some("  Acme Signer \t"), None);
    assert_eq!(caller_label(&c).name, "Acme Signer");
}

#[test]
fn a_blank_product_name_falls_back_to_the_executable_file_name() {
    for product in [None, Some(""), Some("   "), Some("\t\n ")] {
        let c = caller("/opt/acme/bin/acme-cli", product, None);
        assert_eq!(caller_label(&c).name, "acme-cli", "{product:?}");
    }
}

#[test]
fn the_file_name_fallback_keeps_the_extension() {
    let c = caller("/Apps/Acme/Acme.exe", None, None);
    assert_eq!(caller_label(&c).name, "Acme.exe");
}

#[test]
fn a_product_name_of_64_characters_is_kept_whole() {
    let name = "N".repeat(64);
    let c = caller("/opt/x", Some(&name), None);
    assert_eq!(caller_label(&c).name, name);
}

#[test]
fn a_longer_product_name_is_cut_and_ends_with_an_ellipsis() {
    // SPEC: "at most 64 characters with `…`": whether the ellipsis counts
    // towards the 64 is unspecified, so 64 or 65 characters are accepted.
    let name = "N".repeat(100);
    let c = caller("/opt/x", Some(&name), None);
    let got = caller_label(&c).name;
    assert!(got.ends_with('…'), "{got}");
    assert!((64..=65).contains(&got.chars().count()), "{got}");
    assert!(got.starts_with(&"N".repeat(63)), "{got}");
}

#[test]
fn truncation_counts_characters_not_bytes() {
    let name = "é".repeat(100);
    let c = caller("/opt/x", Some(&name), None);
    let got = caller_label(&c).name;
    assert!(got.ends_with('…'), "{got}");
    assert!((64..=65).contains(&got.chars().count()), "{got}");
    assert!(got.starts_with(&"é".repeat(63)), "{got}");

    let exact = "é".repeat(64);
    let c = caller("/opt/x", Some(&exact), None);
    assert_eq!(caller_label(&c).name, exact);
}

#[test]
fn truncation_happens_after_trimming() {
    let name = format!("   {}   ", "N".repeat(64));
    let c = caller("/opt/x", Some(&name), None);
    assert_eq!(caller_label(&c).name, "N".repeat(64));
}

// --- caller_label: detail and verified ---------------------------------------------------------

#[test]
fn an_authenticode_caller_shows_the_signer_subject() {
    let c = caller(
        "/Apps/Acme/acme.exe",
        Some("Acme"),
        authenticode("Acme Corp Ltda"),
    );
    assert_eq!(caller_label(&c), label("Acme", "Acme Corp Ltda", true));
}

#[test]
fn an_apple_caller_shows_identifier_and_team_id() {
    let c = caller(
        "/Applications/Acme.app/Contents/MacOS/Acme",
        Some("Acme"),
        apple("ABCDE12345", "com.acme.signer"),
    );
    assert_eq!(
        caller_label(&c),
        label("Acme", "com.acme.signer (ABCDE12345)", true)
    );
}

#[test]
fn an_unsigned_caller_shows_the_executable_path_and_is_not_verified() {
    let c = caller("/home/ana/tools/acme-cli", Some("Acme"), None);
    assert_eq!(
        caller_label(&c),
        label("Acme", "/home/ana/tools/acme-cli", false)
    );
}

#[test]
fn an_unsigned_caller_without_product_name_is_named_after_its_file() {
    let c = caller("/home/ana/tools/acme-cli", None, None);
    assert_eq!(
        caller_label(&c),
        label("acme-cli", "/home/ana/tools/acme-cli", false)
    );
}

#[test]
fn a_signed_caller_without_product_name_is_named_after_its_file() {
    let c = caller("/Apps/Acme/acme.exe", None, authenticode("Acme Corp"));
    assert_eq!(caller_label(&c), label("acme.exe", "Acme Corp", true));
}

#[test]
fn the_detail_is_not_trimmed_or_altered() {
    let c = caller("/Apps/Acme/acme.exe", None, authenticode("  Acme  Corp "));
    assert_eq!(caller_label(&c).detail, "  Acme  Corp ");
}

// --- consent_key -----------------------------------------------------------------------------------

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
    // SPEC: the key prefixes keep the three families apart.
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
