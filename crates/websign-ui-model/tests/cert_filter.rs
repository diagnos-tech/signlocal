//! SPEC §1.7: the filter field of long lists.

mod common;

use common::candidate;
use websign_core::present::document::DocumentLabel;
use websign_ui_model::certs::{
    Badge, CertRow, FILTER_THRESHOLD, Location, Place, RowStatus, ValidityLabel, matches_filter,
};

fn row(name: &str, issuer: &str, document: Option<DocumentLabel>) -> CertRow {
    CertRow {
        candidate: candidate(1, name),
        name: name.to_owned(),
        badge: Badge::Generic,
        document,
        issuer: issuer.to_owned(),
        location: Location {
            place: Place::Computer,
            via_driver: false,
        },
        validity: ValidityLabel::ExpiresTomorrow,
        status: RowStatus::Usable,
    }
}

fn cpf() -> Option<DocumentLabel> {
    Some(DocumentLabel::Cpf {
        masked: "•••.456.789-••".to_owned(),
        visible: "456 789".to_owned(),
    })
}

fn cnpj() -> Option<DocumentLabel> {
    Some(DocumentLabel::Cnpj {
        formatted: "12.345.678/0001-90".to_owned(),
    })
}

fn national() -> Option<DocumentLabel> {
    Some(DocumentLabel::National {
        masked: "•••••123".to_owned(),
    })
}

#[test]
fn the_filter_appears_above_six_usable_rows() {
    assert_eq!(FILTER_THRESHOLD, 6);
}

#[test]
fn an_empty_query_matches_everything() {
    assert!(matches_filter(&row("Ana", "AC", None), ""));
}

#[test]
fn name_matches_by_substring_ignoring_case() {
    let r = row("Ana Beatriz Souza", "AC Soluti", None);
    for q in ["ana", "ANA", "beatriz sou", "Souza", "a beat"] {
        assert!(matches_filter(&r, q), "{q}");
    }
    assert!(!matches_filter(&r, "carla"));
    assert!(!matches_filter(&r, "souza ana"));
}

#[test]
fn accents_are_ignored_in_both_directions() {
    let plain = row("Alvaro Jose", "AC", None);
    let accented = row("Álvaro José", "AC", None);
    for q in ["álvaro", "alvaro", "ÁLVARO", "josé"] {
        assert!(matches_filter(&plain, q), "plain row, {q}");
        assert!(matches_filter(&accented, q), "accented row, {q}");
    }
    assert!(matches_filter(&row("Conceição", "AC", None), "conceicao"));
}

#[test]
fn issuer_matches_the_same_way() {
    let r = row("Ana", "AC Soluti Múltipla", None);
    assert!(matches_filter(&r, "soluti"));
    assert!(matches_filter(&r, "MULTIPLA"));
    assert!(!matches_filter(&r, "serasa"));
}

#[test]
fn digits_match_the_visible_part_of_a_cpf_in_any_notation() {
    let r = row("Ana", "AC", cpf());
    for q in [
        "456", "789", "456789", "45678", "456.789", "456-789", "456 789", "6 7",
    ] {
        assert!(matches_filter(&r, q), "{q}");
    }
}

#[test]
fn hidden_cpf_digits_never_match() {
    let r = row("Ana", "AC", cpf());
    for q in ["123", "012", "00", "***", "123456789"] {
        assert!(!matches_filter(&r, q), "{q}");
    }
}

#[test]
fn a_cnpj_matches_by_any_digit_run_of_the_formatted_number() {
    let r = row("Clinica", "AC", cnpj());
    for q in [
        "12345678000190",
        "12.345.678/0001-90",
        "0001",
        "345",
        "90",
        "8/0001",
    ] {
        assert!(matches_filter(&r, q), "{q}");
    }
    assert!(!matches_filter(&r, "999"));
}

#[test]
fn a_national_id_matches_only_its_last_three_digits() {
    let r = row("Joao", "AC", national());
    assert!(matches_filter(&r, "123"));
    assert!(matches_filter(&r, "23"));
    assert!(!matches_filter(&r, "12345"));
    assert!(!matches_filter(&r, "456"));
}

#[test]
fn a_row_without_a_document_does_not_match_digits() {
    assert!(!matches_filter(&row("Ana", "AC", None), "456"));
}

#[test]
fn a_query_mixing_letters_and_digits_is_only_a_text_query() {
    let r = row("Ana", "AC", cpf());
    assert!(!matches_filter(&r, "ana 456"));
    let named = row("Sala 456 Clinic", "AC", None);
    assert!(matches_filter(&named, "456"));
}
