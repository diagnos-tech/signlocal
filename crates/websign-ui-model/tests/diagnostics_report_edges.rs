//! SPEC §3.3: normalization and limits of the "Copy diagnostics" text (masked
//! ATR, empty tallies, the 20-error cap).

mod common;

use common::*;
use websign_ui_model::diagnostics::report::{
    CertificateCounts, DeviceLine, ErrorLine, ReportInput, render,
};

fn reader_line(hint_id: Option<&str>, atr: &str) -> String {
    let text = render(&ReportInput {
        devices: vec![DeviceLine::Reader {
            name: s("Identiv uTrust 2700 R"),
            hint_id: hint_id.map(s),
            atr: Some(s(atr)),
            certs: 1,
        }],
        ..empty_sections()
    });
    let line = text.lines().find(|line| line.starts_with("  reader "));
    line.unwrap_or_default().to_owned()
}

#[test]
fn an_unknown_card_is_shown_with_its_historical_bytes_masked() {
    let expected = format!(r#"  reader "Identiv uTrust 2700 R" · atr {ATR_MASKED} · certs 1"#);
    for spelling in [
        ATR,
        "3BD518FF8191FE1FC38073C821100A",
        "3b d5 18 ff 81 91 fe 1f c3 80 73 c8 21 10 0a",
    ] {
        assert_eq!(reader_line(None, spelling), expected, "{spelling}");
    }
}

#[test]
fn a_known_card_is_shown_by_its_device_and_never_by_its_atr() {
    let line = reader_line(Some("safenet-etoken-5110"), ATR);
    assert_eq!(
        line,
        r#"  reader "Identiv uTrust 2700 R" · card safenet-etoken-5110 · certs 1"#
    );
}

#[test]
fn an_atr_that_cannot_be_parsed_is_reduced_to_its_length() {
    let truncated = "3BD518FF8191FE1FC38073C8";
    assert!(reader_line(None, truncated).contains("atr 12 bytes ·"));
}

#[test]
fn text_that_is_not_hex_bytes_is_never_echoed() {
    for odd in ["3BD", "not an atr"] {
        let line = reader_line(None, odd);
        assert!(line.contains("atr unreadable ·"), "{odd}");
        assert!(!line.contains(odd), "{odd}");
    }
}

fn tallies(kinds: Vec<(String, u32)>, keys: Vec<(String, u32)>) -> String {
    let text = render(&ReportInput {
        certificates: CertificateCounts {
            kinds,
            keys,
            ..CertificateCounts::default()
        },
        ..empty_sections()
    });
    let line = text.lines().find(|line| line.starts_with("  kinds: "));
    line.unwrap_or_default().to_owned()
}

#[test]
fn empty_tallies_print_none() {
    assert_eq!(tallies(vec![], vec![]), "  kinds: none · keys: none");
}

#[test]
fn tallies_keep_the_order_they_are_given() {
    let kinds = vec![(s("icp-brasil-a3"), 1), (s("icp-brasil-a1"), 2)];
    let keys = vec![(s("rsa-2048"), 2), (s("ecdsa-p256"), 1)];
    assert_eq!(
        tallies(kinds, keys),
        "  kinds: icp-brasil-a3 1, icp-brasil-a1 2 · keys: rsa-2048 2, ecdsa-p256 1"
    );
}

#[test]
fn only_the_newest_20_errors_are_printed_oldest_first() {
    let errors = (0..25)
        .map(|minute| ErrorLine {
            at: format!("2026-09-29T14:{minute:02}Z"),
            operation: s("sign"),
            code: s("DriverFailure"),
            source: s("pkcs11"),
            native: None,
        })
        .collect();
    let text = render(&ReportInput {
        recent_errors: errors,
        ..empty_sections()
    });
    let printed: Vec<&str> = text
        .lines()
        .skip_while(|line| !line.starts_with("recent errors"))
        .skip(1)
        .collect();
    assert_eq!(printed.len(), 20);
    assert_eq!(printed[0], "  2026-09-29T14:05Z sign DriverFailure pkcs11");
    assert_eq!(printed[19], "  2026-09-29T14:24Z sign DriverFailure pkcs11");
}
