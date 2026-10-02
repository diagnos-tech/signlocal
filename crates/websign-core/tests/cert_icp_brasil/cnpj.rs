//! CNPJ.

use super::*;

#[test]
fn reads_the_cnpj_in_every_accepted_string_type() {
    for name in ["icp-pj-a1", "icp-pj-printable", "icp-pj-utf8", "icp-pj-ia5"] {
        assert_eq!(
            icp(name),
            company(Some(CNPJ), Some(RESPONSIBLE_CPF)),
            "{name}"
        );
    }
}

#[test]
fn rejects_a_cnpj_that_is_not_exactly_14_digits() {
    for name in ["icp-pj-cnpj-13", "icp-pj-cnpj-15", "icp-pj-cnpj-letter"] {
        assert_eq!(icp(name), company(None, Some(RESPONSIBLE_CPF)), "{name}");
    }
}

#[test]
fn rejects_an_all_zero_cnpj() {
    assert_eq!(
        icp("icp-pj-cnpj-zeros"),
        company(None, Some(RESPONSIBLE_CPF))
    );
}

#[test]
fn a_company_without_a_responsible_person_has_no_cpf() {
    assert_eq!(icp("icp-pj-no-responsible"), company(Some(CNPJ), None));
}

#[test]
fn a_person_certificate_has_no_cnpj() {
    assert_eq!(icp("icp-pf-a3").cnpj, None);
}
