//! SPEC §10 (`docs/ux.md` §5.2, vectors §16.3): `present::holder`.

mod common;

use common::{blank_info, info};
use websign_core::CertInfo;
use websign_core::IcpBrasil;
use websign_core::present::holder::{display_name, title_case};

fn with_cn(cn: &str) -> CertInfo {
    let mut info = blank_info();
    info.subject.common_name = Some(cn.to_owned());
    info
}

/// An ICP-Brasil certificate the way the cert reader would summarize it.
fn icp_with_cn(cn: &str) -> CertInfo {
    let mut info = with_cn(cn);
    info.icp_brasil = Some(IcpBrasil {
        level: None,
        holder_name: Some(cn.rsplit_once(':').map_or(cn, |(name, _)| name).to_owned()),
        cpf: None,
        cnpj: None,
    });
    info
}

// --- ux.md §16.3 -------------------------------------------------------------------------------

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

// --- display_name with real certificates ---------------------------------------------------------

#[test]
fn reads_the_icp_brasil_holder_from_a_certificate() {
    assert_eq!(display_name(&info("icp-pf-a3")), "Ana Beatriz Souza");
    assert_eq!(display_name(&info("icp-pj-a1")), "Empresa Teste Ltda");
    assert_eq!(display_name(&info("icp-cn-accented")), "José D'Ávila");
    // SPEC: "R2:D2" is one word, so only its first letter stays upper case.
    assert_eq!(display_name(&info("icp-cn-nested-colon")), "R2:d2");
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

// --- fingerprint fallback and given name + surname -----------------------------------------------

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
fn a_common_name_or_organization_beats_given_name_and_surname() {
    let mut info = blank_info();
    info.subject.given_name = Some("Maria".to_owned());
    info.subject.surname = Some("Silva".to_owned());
    info.subject.organization = Some("Clinica Sol".to_owned());
    assert_eq!(display_name(&info), "Clinica Sol");
    info.subject.common_name = Some("Dra Maria S".to_owned());
    assert_eq!(display_name(&info), "Dra Maria S");
}

#[test]
fn blank_candidates_are_skipped_before_given_name_and_surname() {
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

// --- title_case --------------------------------------------------------------------------------

#[test]
fn title_cases_every_word() {
    assert_eq!(
        title_case("JOAO DA SILVA DOS SANTOS"),
        "Joao da Silva dos Santos"
    );
    assert_eq!(title_case("ana beatriz souza"), "Ana Beatriz Souza");
    assert_eq!(title_case("aNa bEaTrIz"), "Ana Beatriz");
    assert_eq!(title_case("A"), "A");
    assert_eq!(title_case(""), "");
}

#[test]
fn particles_are_lowercase_unless_first() {
    for particle in [
        "DA", "DAS", "DE", "DI", "DO", "DOS", "DU", "E", "DEL", "LA", "VAN", "VON", "Y",
    ] {
        let input = format!("MARIA {particle} SILVA");
        let expected = format!("Maria {} Silva", particle.to_lowercase());
        assert_eq!(title_case(&input), expected, "{particle} in the middle");

        let first = format!("{particle} SILVA");
        let mut chars = particle.chars();
        let capitalized: String = chars
            .next()
            .map(|c| {
                c.to_uppercase()
                    .chain(chars.flat_map(char::to_lowercase))
                    .collect()
            })
            .unwrap_or_default();
        assert_eq!(
            title_case(&first),
            format!("{capitalized} Silva"),
            "{particle} first"
        );
    }
}

#[test]
fn words_that_only_contain_a_particle_are_not_particles() {
    assert_eq!(title_case("MARIA DAVI SILVA"), "Maria Davi Silva");
    assert_eq!(title_case("MARIA DOMINGOS"), "Maria Domingos");
    assert_eq!(title_case("MARIA DER BERG"), "Maria Der Berg");
    assert_eq!(title_case("MARIA ELA"), "Maria Ela");
}

#[test]
fn spanish_and_dutch_particles_follow_the_same_rule() {
    assert_eq!(title_case("JUAN DEL RIO"), "Juan del Rio");
    assert_eq!(
        title_case("CARLOS DE LA CRUZ Y SOTO"),
        "Carlos de la Cruz y Soto"
    );
    assert_eq!(title_case("LUDWIG VAN BEETHOVEN"), "Ludwig van Beethoven");
    assert_eq!(title_case("OTTO VON BISMARCK"), "Otto von Bismarck");
}

#[test]
fn company_suffixes_keep_their_written_form() {
    assert_eq!(title_case("PADARIA CENTRAL ME"), "Padaria Central ME");
    assert_eq!(title_case("OFICINA JOSE EPP"), "Oficina Jose EPP");
    assert_eq!(title_case("LOJA AZUL EIRELI"), "Loja Azul EIRELI");
    assert_eq!(title_case("BANCO XYZ S.A."), "Banco Xyz S.A.");
    assert_eq!(title_case("COMERCIO S/A"), "Comercio S/A");
}

#[test]
fn ltda_becomes_ltda() {
    assert_eq!(
        title_case("CLINICA SOUZA IMAGEM LTDA"),
        "Clinica Souza Imagem Ltda"
    );
    assert_eq!(title_case("LTDA"), "Ltda");
}

#[test]
fn unicode_letters_are_cased_by_unicode_rules() {
    assert_eq!(title_case("AÇÃO"), "Ação");
    assert_eq!(title_case("ÇÃO"), "Ção");
    assert_eq!(title_case("ÁGATA ÉLIA ÖZDEMIR"), "Ágata Élia Özdemir");
    assert_eq!(title_case("ÑANDÚ"), "Ñandú");
}

#[test]
fn hyphenated_parts_and_apostrophes_capitalize_each_piece() {
    assert_eq!(title_case("MARIA-CLARA"), "Maria-Clara");
    assert_eq!(
        title_case("JEAN-PIERRE DE LA CRUZ"),
        "Jean-Pierre de la Cruz"
    );
    assert_eq!(title_case("D'ÁVILA"), "D'Ávila");
    assert_eq!(title_case("O'BRIEN"), "O'Brien");
    assert_eq!(title_case("JOSÉ D'ÁVILA"), "José D'Ávila");
    assert_eq!(
        title_case("ANA-MARIA D'OLIVEIRA-LIMA"),
        "Ana-Maria D'Oliveira-Lima"
    );
}

#[test]
fn separators_are_kept_exactly() {
    assert_eq!(title_case("ANA  BEATRIZ"), "Ana  Beatriz");
    assert_eq!(title_case(" ANA BEATRIZ "), " Ana Beatriz ");
    assert_eq!(title_case("ANA\tBEATRIZ"), "Ana\tBeatriz");
    assert_eq!(title_case("   "), "   ");
}

#[test]
fn digits_and_punctuation_stay_where_they_are() {
    assert_eq!(title_case("SALA 12"), "Sala 12");
    assert_eq!(title_case("12 BARRA 34"), "12 Barra 34");
    assert_eq!(title_case("R2:D2"), "R2:d2");
    assert_eq!(title_case("A.B. SILVA"), "A.b. Silva");
}

#[test]
fn title_case_is_idempotent_on_its_own_output() {
    for input in [
        "JOAO DA SILVA DOS SANTOS",
        "CLINICA SOUZA IMAGEM LTDA",
        "JOSÉ D'ÁVILA",
        "MARIA-CLARA DE LA CRUZ",
        "E SOUZA COMERCIO ME",
    ] {
        let once = title_case(input);
        assert_eq!(title_case(&once), once, "{input}");
    }
}

#[test]
fn never_panics_on_odd_text() {
    let long = "A".repeat(10_000);
    let inputs = [
        "\u{0}",
        "\u{202e}ABC",
        "ǅ ǆ ǈ",
        "ß",
        "İSTANBUL",
        "ΣΊΣΥΦΟΣ",
        "\u{1f4a9} ABC",
        "-",
        "'",
        "--''",
        &long,
    ];
    for input in inputs {
        let _ = title_case(input);
    }
}
