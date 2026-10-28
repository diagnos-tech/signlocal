//! `title_case` word by word.

use super::*;

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
fn company_suffixes_take_their_official_spelling() {
    assert_eq!(title_case("PADARIA CENTRAL ME"), "Padaria Central ME");
    assert_eq!(title_case("OFICINA JOSE EPP"), "Oficina Jose EPP");
    assert_eq!(title_case("LOJA AZUL EIRELI"), "Loja Azul EIRELI");
    assert_eq!(title_case("BANCO XYZ S.A."), "Banco Xyz S.A.");
    assert_eq!(title_case("COMERCIO S/A"), "Comercio S/A");
    assert_eq!(title_case("Padaria me"), "Padaria ME");
    assert_eq!(title_case("banco s.a."), "Banco S.A.");
}

#[test]
fn particles_match_in_any_case_and_only_as_whole_words() {
    assert_eq!(title_case("Maria Da Silva"), "Maria da Silva");
    assert_eq!(title_case("MARIA DA-SILVA"), "Maria Da-Silva");
    assert_eq!(title_case("  DA SILVA"), "  Da Silva");
}

#[test]
fn a_decomposed_accent_does_not_start_a_new_part() {
    assert_eq!(title_case("A\u{301}VILA"), "A\u{301}vila");
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
    assert_eq!(title_case("R2:D2"), "R2:D2");
    assert_eq!(title_case("A.B. SILVA"), "A.B. Silva");
    assert_eq!(title_case("3M DO BRASIL"), "3M do Brasil");
    assert_eq!(title_case("JOAO (SP)"), "Joao (Sp)");
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
