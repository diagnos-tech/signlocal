//! The §16.4 vectors, priority and CPF shapes.

use super::*;

#[test]
fn masks_the_cpf_keeping_digits_four_to_nine() {
    assert_eq!(
        display_document(&icp(Some("12345678909"), None)),
        Some(cpf_label(
            "\u{2022}\u{2022}\u{2022}.456.789-\u{2022}\u{2022}",
            "456 789"
        ))
    );
}

#[test]
fn the_mask_uses_the_bullet_character_u2022() {
    let Some(DocumentLabel::Cpf { masked, .. }) = display_document(&icp(Some("12345678909"), None))
    else {
        panic!("expected a CPF label");
    };
    assert_eq!(masked, "•••.456.789-••");
    assert_eq!(masked.chars().filter(|&c| c == '\u{2022}').count(), 5);
}

#[test]
fn formats_the_cnpj_in_full() {
    assert_eq!(
        display_document(&icp(None, Some("12345678000190"))),
        Some(cnpj_label("12.345.678/0001-90"))
    );
}

#[test]
fn masks_an_etsi_national_identifier_keeping_the_last_three_characters() {
    assert_eq!(
        display_document(&with_serial("IDCPT-12345123")),
        Some(national("•••••123"))
    );
}

#[test]
fn a_certificate_with_no_document_has_no_label() {
    assert_eq!(display_document(&blank_info()), None);
    assert_eq!(display_document(&icp(None, None)), None);
}

#[test]
fn cnpj_wins_over_the_cpf_and_the_serial_number() {
    // A certificate with a CNPJ belongs to the company; its CPF is the
    // person responsible for it.
    let mut info = icp(Some("12345678909"), Some("12345678000190"));
    info.subject.serial_number = Some("IDCPT-12345123".to_owned());
    assert_eq!(
        display_document(&info),
        Some(cnpj_label("12.345.678/0001-90"))
    );
}

#[test]
fn cpf_wins_over_the_serial_number() {
    let mut info = icp(Some("12345678909"), None);
    info.subject.serial_number = Some("IDCPT-12345123".to_owned());
    assert_eq!(
        display_document(&info),
        Some(cpf_label("•••.456.789-••", "456 789"))
    );
}

#[test]
fn cnpj_wins_over_the_serial_number() {
    let mut info = icp(None, Some("12345678000190"));
    info.subject.serial_number = Some("IDCPT-12345123".to_owned());
    assert_eq!(
        display_document(&info),
        Some(cnpj_label("12.345.678/0001-90"))
    );
}

#[test]
fn the_serial_number_is_used_when_the_icp_data_is_absent() {
    let mut info = icp(None, None);
    info.subject.serial_number = Some("PNOPT-987654321".to_owned());
    assert_eq!(display_document(&info), Some(national("•••••321")));
}

#[test]
fn a_cpf_that_is_not_eleven_digits_falls_through_to_the_next_document() {
    // The field is public: anyone can set it (§11).
    for bad in [
        "123",
        "1234567890",
        "123456789012",
        "1234567890A",
        "١٢٣٤٥٦٧٨٩٠١",
        "",
    ] {
        assert_eq!(display_document(&icp(Some(bad), None)), None, "{bad:?}");
        let mut with_serial = icp(Some(bad), None);
        with_serial.subject.serial_number = Some("PNOPT-987654321".to_owned());
        assert_eq!(
            display_document(&with_serial),
            Some(national("•••••321")),
            "{bad:?}"
        );
    }
}

#[test]
fn a_cnpj_that_is_not_fourteen_digits_falls_through_to_the_next_document() {
    for bad in ["1234567800019", "123456780001901", "1234567800019X", ""] {
        assert_eq!(display_document(&icp(None, Some(bad))), None, "{bad:?}");
        assert_eq!(
            display_document(&icp(Some("12345678909"), Some(bad))),
            Some(cpf_label("•••.456.789-••", "456 789")),
            "{bad:?}"
        );
    }
}

#[test]
fn masks_other_cpfs_by_position() {
    let cases = [
        ("98765432100", "•••.654.321-••", "654 321"),
        ("00000000000", "•••.000.000-••", "000 000"),
        ("11122233344", "•••.222.333-••", "222 333"),
        ("01234567890", "•••.345.678-••", "345 678"),
    ];
    for (cpf, masked, visible) in cases {
        assert_eq!(
            display_document(&icp(Some(cpf), None)),
            Some(cpf_label(masked, visible)),
            "{cpf}"
        );
    }
}
