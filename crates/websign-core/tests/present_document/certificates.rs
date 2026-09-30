//! Labels of the OpenSSL fixtures.

use super::*;

#[test]
fn reads_the_cpf_of_an_icp_brasil_person_certificate() {
    assert_eq!(
        display_document(&info("icp-pf-a3")),
        Some(cpf_label("•••.456.789-••", "456 789"))
    );
}

#[test]
fn a_company_certificate_shows_the_cnpj_not_the_responsible_persons_cpf() {
    for name in ["icp-pj-a1", "icp-pj-printable", "icp-pj-utf8", "icp-pj-ia5"] {
        assert_eq!(
            display_document(&info(name)),
            Some(cnpj_label("12.345.678/0001-95")),
            "{name}"
        );
    }
}

#[test]
fn a_company_certificate_with_an_unusable_cnpj_falls_back_to_the_masked_cpf() {
    for name in ["icp-pj-cnpj-13", "icp-pj-cnpj-zeros", "icp-pj-cnpj-letter"] {
        assert_eq!(
            display_document(&info(name)),
            Some(cpf_label("•••.654.321-••", "654 321")),
            "{name}"
        );
    }
}

#[test]
fn a_company_certificate_without_a_responsible_person_shows_the_cnpj() {
    assert_eq!(
        display_document(&info("icp-pj-no-responsible")),
        Some(cnpj_label("12.345.678/0001-95"))
    );
}

#[test]
fn reads_the_serial_number_of_a_european_certificate() {
    assert_eq!(
        display_document(&info("dn-pii")),
        Some(national("•••••123"))
    );
    assert_eq!(
        display_document(&info("dn-pii-only")),
        Some(national("•••••123"))
    );
}

#[test]
fn uses_the_first_readable_serial_number() {
    // The first serialNumber of this certificate is unreadable, the second is ETSI-shaped.
    assert_eq!(
        display_document(&info("dn-pii-unreadable-first")),
        Some(national("•••••222"))
    );
}

#[test]
fn ordinary_certificates_have_no_document() {
    for name in [
        "rsa2048",
        "p256",
        "ca",
        "dn-utf8",
        "qc-esign-sscd",
        "icp-lookalike",
    ] {
        assert_eq!(display_document(&info(name)), None, "{name}");
    }
}
