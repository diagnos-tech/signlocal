//! The §16.3 vectors and the name candidates.

use super::*;

#[test]
fn shows_the_holder_without_the_document_suffix_in_title_case() {
    let cases = [
        ("ANA BEATRIZ SOUZA:12345678909", "Ana Beatriz Souza"),
        (
            "JOAO DA SILVA DOS SANTOS:98765432100",
            "Joao da Silva dos Santos",
        ),
        (
            "CLINICA SOUZA IMAGEM LTDA:12345678000190",
            "Clinica Souza Imagem Ltda",
        ),
    ];
    for (cn, expected) in cases {
        assert_eq!(display_name(&icp_with_cn(cn)), expected, "{cn}");
    }
}

#[test]
fn a_leading_particle_stays_capitalized_and_a_company_suffix_is_kept() {
    assert_eq!(
        display_name(&with_cn("E SOUZA COMERCIO ME")),
        "E Souza Comercio ME"
    );
}

#[test]
fn a_name_that_is_not_all_caps_is_kept_as_it_is() {
    assert_eq!(
        display_name(&with_cn("Marta Sofia Carvalho")),
        "Marta Sofia Carvalho"
    );
    for cn in [
        "marta sofia carvalho",
        "MARTA sofia",
        "Marta SOFIA",
        "McDonald",
    ] {
        assert_eq!(display_name(&with_cn(cn)), cn, "{cn}");
    }
}

#[test]
fn keeps_accents_apostrophes_and_hyphens() {
    assert_eq!(display_name(&with_cn("JOSÉ D'ÁVILA")), "José D'Ávila");
    assert_eq!(
        display_name(&with_cn("MARIA-CLARA LIMA")),
        "Maria-Clara Lima"
    );
    assert_eq!(
        display_name(&with_cn("JOSÉ AÇAÍ DA SILVA")),
        "José Açaí da Silva"
    );
}

#[test]
fn reads_the_icp_brasil_holder_from_a_certificate() {
    assert_eq!(display_name(&info("icp-pf-a3")), "Ana Beatriz Souza");
    assert_eq!(display_name(&info("icp-pj-a1")), "Empresa Teste Ltda");
    assert_eq!(display_name(&info("icp-cn-accented")), "José D'Ávila");
    // Punctuation starts a new part of the word.
    assert_eq!(display_name(&info("icp-cn-nested-colon")), "R2:D2");
}

#[test]
fn falls_back_to_the_common_name_then_the_organization() {
    assert_eq!(display_name(&info("dn-utf8")), "José Açaí da Silva");
    assert_eq!(display_name(&info("dn-printable")), "Printable Name 123");
    // Subject with only an organization.
    assert_eq!(display_name(&info("dn-org-only")), "Only Org Name");
    // Not all upper case: untouched.
    assert_eq!(display_name(&info("p256")), "Fixture p256");
}

#[test]
fn title_case_is_applied_to_an_all_caps_organization_too() {
    let mut info = blank_info();
    info.subject.organization = Some("ACME COMERCIO DE PECAS LTDA".to_owned());
    assert_eq!(display_name(&info), "Acme Comercio de Pecas Ltda");
}

#[test]
fn a_name_without_letters_is_left_alone() {
    for cn in ["12345", "123-456", "+++", "0"] {
        assert_eq!(display_name(&with_cn(cn)), cn, "{cn}");
    }
}

#[test]
fn a_name_with_digits_and_only_upper_case_letters_is_converted() {
    assert_eq!(display_name(&with_cn("SALA 12 LTDA")), "Sala 12 Ltda");
}

#[test]
fn uses_given_name_and_surname_when_no_candidate_was_found() {
    let mut info = blank_info();
    info.subject.given_name = Some("Maria".to_owned());
    info.subject.surname = Some("Silva".to_owned());
    assert_eq!(display_name(&info), "Maria Silva");
}

#[test]
fn given_name_and_surname_in_capitals_are_converted_to_title_case() {
    let mut caps = blank_info();
    caps.subject.given_name = Some("MARIA".to_owned());
    caps.subject.surname = Some("SILVA".to_owned());
    assert_eq!(display_name(&caps), "Maria Silva");
    assert_eq!(display_name(&info("dn-pii-only")), "Maria Silva");
}

#[test]
fn given_name_and_surname_beat_the_organization_but_not_the_common_name() {
    // A personal certificate's O is usually the employer (ux.md §5.2).
    let mut info = blank_info();
    info.subject.given_name = Some("Maria".to_owned());
    info.subject.surname = Some("Silva".to_owned());
    info.subject.organization = Some("Clinica Sol".to_owned());
    assert_eq!(display_name(&info), "Maria Silva");
    assert_eq!(info.display_name(), "Maria Silva");
    info.subject.common_name = Some("Dra Maria S".to_owned());
    assert_eq!(display_name(&info), "Dra Maria S");
}

#[test]
fn given_name_and_surname_are_trimmed_and_both_required() {
    let mut info = blank_info();
    info.subject.given_name = Some("  Maria ".to_owned());
    info.subject.surname = Some(" Silva\t".to_owned());
    assert_eq!(display_name(&info), "Maria Silva");
    info.subject.surname = Some("   ".to_owned());
    info.subject.organization = Some("Clinica Sol".to_owned());
    assert_eq!(display_name(&info), "Clinica Sol");
}

#[test]
fn control_and_bidi_characters_never_reach_the_name() {
    let rlo = with_cn("Invoice\u{202e}gpj.exe");
    assert_eq!(display_name(&rlo), "Invoicegpj.exe");
    let tab = with_cn("ANA\tSOUZA\u{0}");
    assert_eq!(display_name(&tab), "Ana Souza");
    let only_controls = with_cn("\u{202e}\u{200f}");
    assert_eq!(
        display_name(&only_controls),
        &only_controls.fingerprint.to_hex()[..16]
    );
}

#[test]
fn blank_candidates_are_skipped() {
    let mut info = blank_info();
    info.subject.common_name = Some("   ".to_owned());
    info.subject.organization = Some(String::new());
    info.subject.given_name = Some("Maria".to_owned());
    info.subject.surname = Some("Silva".to_owned());
    assert_eq!(display_name(&info), "Maria Silva");
}

#[test]
fn falls_back_to_the_fingerprint_prefix_without_both_names() {
    let expected = &blank_info().fingerprint.to_hex()[..16];
    assert_eq!(expected.len(), 16);

    assert_eq!(display_name(&blank_info()), expected);

    let mut only_given = blank_info();
    only_given.subject.given_name = Some("Maria".to_owned());
    assert_eq!(display_name(&only_given), expected);

    let mut only_surname = blank_info();
    only_surname.subject.surname = Some("Silva".to_owned());
    assert_eq!(display_name(&only_surname), expected);
}

#[test]
fn the_fingerprint_fallback_is_not_title_cased() {
    // Lower-case hex letters: not "all uppercase", so left as they are.
    let mut info = blank_info();
    info.fingerprint = websign_core::Fingerprint::from_bytes([0xCD; 32]);
    assert_eq!(display_name(&info), "cdcdcdcdcdcdcdcd");
}

#[test]
fn a_fingerprint_of_only_digits_is_left_as_it_is() {
    let mut info = blank_info();
    info.fingerprint = websign_core::Fingerprint::from_bytes([0x12; 32]);
    assert_eq!(display_name(&info), "1212121212121212");
}
