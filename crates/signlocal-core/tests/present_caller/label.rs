//! `caller_label`: name, detail and verified.

use super::*;

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
    // 64 characters in total: 63 kept and the ellipsis.
    for len in [65, 100] {
        let c = caller("/opt/x", Some(&"N".repeat(len)), None);
        assert_eq!(
            caller_label(&c).name,
            format!("{}…", "N".repeat(63)),
            "{len}"
        );
    }
}

#[test]
fn truncation_counts_characters_not_bytes() {
    let name = "é".repeat(100);
    let c = caller("/opt/x", Some(&name), None);
    assert_eq!(caller_label(&c).name, format!("{}…", "é".repeat(63)));

    let exact = "é".repeat(64);
    let c = caller("/opt/x", Some(&exact), None);
    assert_eq!(caller_label(&c).name, exact);
}

#[test]
fn whitespace_before_the_cut_is_dropped() {
    let name = format!("{} {}", "N".repeat(62), "M".repeat(10));
    let c = caller("/opt/x", Some(&name), None);
    assert_eq!(caller_label(&c).name, format!("{}…", "N".repeat(62)));
}

#[test]
fn control_and_bidi_characters_are_removed_from_what_is_shown() {
    let c = caller(
        "/tmp/invoice\u{202e}gpj.exe",
        Some("Acme\u{202e}\nSigner"),
        authenticode("Acme\u{200f} Corp"),
    );
    assert_eq!(caller_label(&c), label("Acme Signer", "Acme Corp", true));
    let unsigned = caller("/tmp/invoice\u{202e}gpj.exe", None, None);
    assert_eq!(
        caller_label(&unsigned),
        label("invoicegpj.exe", "/tmp/invoicegpj.exe", false)
    );
    let invisible = caller("/opt/acme-cli", Some("\u{202e}\u{2066}"), None);
    assert_eq!(caller_label(&invisible).name, "acme-cli");
}

#[test]
fn a_path_without_a_file_name_is_named_by_the_whole_path() {
    for path in ["/", ".."] {
        let c = caller(path, None, None);
        assert_eq!(caller_label(&c).name, path, "{path}");
    }
}

#[test]
fn truncation_happens_after_trimming() {
    let name = format!("   {}   ", "N".repeat(64));
    let c = caller("/opt/x", Some(&name), None);
    assert_eq!(caller_label(&c).name, "N".repeat(64));
}

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
