use super::{icp_info, info_with};
use crate::present::document::{DocumentLabel, display_document};
use crate::testkit::*;

fn national(serial: &str) -> Option<DocumentLabel> {
    display_document(&info_with(&[rdn("2.5.4.5", PRINTABLE, serial.as_bytes())]))
}

#[test]
fn cpf_is_masked_and_readable_aloud() {
    let info = icp_info("ANA:12345678909", Some("12345678909"), None);
    assert_eq!(
        display_document(&info),
        Some(DocumentLabel::Cpf {
            masked: "•••.456.789-••".into(),
            visible: "456 789".into(),
        })
    );
}

#[test]
fn a_company_shows_its_cnpj_in_full_even_with_a_cpf() {
    let cnpj = Some(DocumentLabel::Cnpj {
        formatted: "12.345.678/0001-90".into(),
    });
    let company = icp_info("CLINICA", None, Some("12345678000190"));
    assert_eq!(display_document(&company), cnpj);
    let with_officer = icp_info("CLINICA", Some("12345678909"), Some("12345678000190"));
    assert_eq!(display_document(&with_officer), cnpj);
}

#[test]
fn etsi_serial_numbers_keep_only_the_last_three_characters() {
    for (serial, masked) in [
        ("IDCPT-12345123", "•••••123"),
        ("PNOPT-987", "•••••987"),
        ("TINIT-RSSMRA80A01H501U", "•••••01U"),
    ] {
        assert_eq!(
            national(serial),
            Some(DocumentLabel::National {
                masked: masked.into()
            }),
            "{serial}"
        );
    }
}

#[test]
fn other_serial_numbers_and_certificates_show_nothing() {
    for serial in [
        "IDCPT-12",
        "idcpt-12345",
        "IDC-12345",
        "12345123",
        "IDCPTX-12345",
        "ABCPT-12345",
        "-12345",
    ] {
        assert_eq!(national(serial), None, "{serial}");
    }
    assert_eq!(display_document(&info_with(&[rdn(CN, UTF8, b"X")])), None);
}
