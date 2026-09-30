use super::{icp_info, info_with};
use crate::present::holder::{display_name, title_case};
use crate::testkit::*;

#[test]
fn title_case_vectors() {
    let cases = [
        ("ANA BEATRIZ SOUZA", "Ana Beatriz Souza"),
        ("JOAO DA SILVA DOS SANTOS", "Joao da Silva dos Santos"),
        ("CLINICA SOUZA IMAGEM LTDA", "Clinica Souza Imagem Ltda"),
        ("E SOUZA COMERCIO ME", "E Souza Comercio ME"),
        ("JOSÉ D'ÁVILA", "José D'Ávila"),
        ("MARIA-CLARA DE SOUZA", "Maria-Clara de Souza"),
        ("SÃO JOÃO S.A.", "São João S.A."),
        ("FOO S/A EPP EIRELI", "Foo S/A EPP EIRELI"),
        ("AÇÃO  DE\tVAN", "Ação  de\tvan"),
        ("DE LA CRUZ Y SOUZA", "De la Cruz y Souza"),
        ("", ""),
    ];
    for (input, expected) in cases {
        assert_eq!(title_case(input), expected, "{input:?}");
    }
}

#[test]
fn icp_holder_without_document_suffix_is_title_cased() {
    let info = icp_info("ANA BEATRIZ SOUZA:12345678909", None, None);
    assert_eq!(display_name(&info), "Ana Beatriz Souza");
}

#[test]
fn only_all_uppercase_names_are_rewritten() {
    let mixed = info_with(&[rdn(CN, UTF8, b"Marta Sofia Carvalho")]);
    assert_eq!(display_name(&mixed), "Marta Sofia Carvalho");
    let odd = info_with(&[rdn(CN, UTF8, b"MARTA de Souza")]);
    assert_eq!(display_name(&odd), "MARTA de Souza");
    let upper = info_with(&[rdn(CN, UTF8, b"CLINICA SOUZA IMAGEM LTDA")]);
    assert_eq!(display_name(&upper), "Clinica Souza Imagem Ltda");
}

#[test]
fn given_name_and_surname_replace_the_fingerprint_fallback() {
    let info = info_with(&[
        rdn("2.5.4.42", UTF8, b"MARTA"),
        rdn("2.5.4.4", UTF8, b"CARVALHO"),
    ]);
    assert_eq!(display_name(&info), "Marta Carvalho");
    let only_given = info_with(&[rdn("2.5.4.42", UTF8, b"MARTA")]);
    assert_eq!(display_name(&only_given), only_given.display_name());
    let with_cn = info_with(&[
        rdn(CN, UTF8, b"Dr. Marta"),
        rdn("2.5.4.42", UTF8, b"X"),
        rdn("2.5.4.4", UTF8, b"Y"),
    ]);
    assert_eq!(display_name(&with_cn), "Dr. Marta");
}

#[test]
fn distinguished_name_reads_the_personal_attributes() {
    let info = info_with(&[
        rdn("2.5.4.42", UTF8, b"Marta"),
        rdn("2.5.4.4", UTF8, b"Carvalho"),
        rdn("2.5.4.5", PRINTABLE, b"IDCPT-12345123"),
    ]);
    assert_eq!(info.subject.given_name.as_deref(), Some("Marta"));
    assert_eq!(info.subject.surname.as_deref(), Some("Carvalho"));
    assert_eq!(
        info.subject.serial_number.as_deref(),
        Some("IDCPT-12345123")
    );
}
