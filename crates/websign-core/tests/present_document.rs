//! SPEC §11 (`docs/ux.md` §5.5, vectors §16.4): `present::document`.

mod common;

use common::{blank_info, cert_names, info};
use websign_core::present::document::{DocumentLabel, display_document};
use websign_core::{CertInfo, IcpBrasil};

fn icp(cpf: Option<&str>, cnpj: Option<&str>) -> CertInfo {
    let mut info = blank_info();
    info.icp_brasil = Some(IcpBrasil {
        level: None,
        holder_name: None,
        cpf: cpf.map(str::to_owned),
        cnpj: cnpj.map(str::to_owned),
    });
    info
}

fn with_serial(serial: &str) -> CertInfo {
    let mut info = blank_info();
    info.subject.serial_number = Some(serial.to_owned());
    info
}

fn cpf_label(masked: &str, visible: &str) -> DocumentLabel {
    DocumentLabel::Cpf {
        masked: masked.to_owned(),
        visible: visible.to_owned(),
    }
}

fn national(masked: &str) -> DocumentLabel {
    DocumentLabel::National {
        masked: masked.to_owned(),
    }
}

fn cnpj_label(formatted: &str) -> DocumentLabel {
    DocumentLabel::Cnpj {
        formatted: formatted.to_owned(),
    }
}

// --- ux.md §16.4 -------------------------------------------------------------------------------

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

// --- priority ------------------------------------------------------------------------------------

#[test]
fn cpf_wins_over_cnpj_and_the_serial_number() {
    let mut info = icp(Some("12345678909"), Some("12345678000190"));
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
    // SPEC: the field is public data anyone can set (§6.3 `masked_cpf`), so a
    // malformed one is not shown; the next rule applies.
    for bad in [
        "123",
        "1234567890",
        "123456789012",
        "1234567890A",
        "١٢٣٤٥٦٧٨٩٠١",
        "",
    ] {
        assert_eq!(display_document(&icp(Some(bad), None)), None, "{bad:?}");
        assert_eq!(
            display_document(&icp(Some(bad), Some("12345678000190"))),
            Some(cnpj_label("12.345.678/0001-90")),
            "{bad:?}"
        );
    }
}

#[test]
fn a_cnpj_that_is_not_fourteen_digits_is_not_shown() {
    for bad in ["1234567800019", "123456780001901", "1234567800019X", ""] {
        assert_eq!(display_document(&icp(None, Some(bad))), None, "{bad:?}");
    }
}

// --- CPF shapes ----------------------------------------------------------------------------------

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

// --- ETSI serial numbers -------------------------------------------------------------------------

#[test]
fn accepts_the_etsi_prefixes() {
    let cases = [
        ("IDCPT-12345123", "•••••123"),
        ("PNOPT-1234", "•••••234"),
        ("IDCES-123", "•••••123"),
        ("TINIT-ABC", "•••••ABC"),
        ("PNODE-12345678", "•••••678"),
        ("IDCPT-ABCDEFGH-1234", "•••••234"),
        ("ZZZZZ-999", "•••••999"),
    ];
    for (serial, masked) in cases {
        assert_eq!(
            display_document(&with_serial(serial)),
            Some(national(masked)),
            "{serial}"
        );
    }
}

#[test]
fn needs_at_least_three_characters_after_the_dash() {
    for serial in ["IDCES-", "IDCES-1", "IDCES-12"] {
        assert_eq!(display_document(&with_serial(serial)), None, "{serial}");
    }
    assert!(display_document(&with_serial("IDCES-123")).is_some());
}

#[test]
fn rejects_serial_numbers_without_the_etsi_shape() {
    for serial in [
        "",
        "12345",
        "IDCPT12345",
        "IDCP-12345",
        "IDCPTX-12345",
        "idcpt-12345",
        "IdCPT-12345",
        "IDCPT_12345",
        "IDC1T-12345",
        "12CPT-12345",
        " IDCPT-12345",
        "-12345",
        "IDCPT",
        "ÍDCPT-12345",
        "IDCPT–12345",
    ] {
        assert_eq!(display_document(&with_serial(serial)), None, "{serial:?}");
    }
}

#[test]
fn the_mask_never_shows_more_than_the_last_three_characters() {
    let Some(DocumentLabel::National { masked }) = display_document(&with_serial("IDCPT-12345123"))
    else {
        panic!("expected a national label");
    };
    assert_eq!(masked.chars().count(), 8);
    assert!(!masked.contains("12345"));
    assert!(!masked.contains("IDCPT"));
}

// --- real certificates ---------------------------------------------------------------------------

#[test]
fn reads_the_cpf_of_an_icp_brasil_person_certificate() {
    assert_eq!(
        display_document(&info("icp-pf-a3")),
        Some(cpf_label("•••.456.789-••", "456 789"))
    );
}

#[test]
fn a_company_certificate_shows_the_responsible_persons_cpf_first() {
    // `icp-pj-a1` has the CPF of the person responsible (otherName .3.4) and a
    // CNPJ; SPEC §11 lists the CPF first.
    assert_eq!(
        display_document(&info("icp-pj-a1")),
        Some(cpf_label("•••.654.321-••", "654 321"))
    );
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

// --- privacy -------------------------------------------------------------------------------------

/// Whether `text` contains a run of `len` or more ASCII digits.
fn has_digit_run(text: &str, len: usize) -> bool {
    let mut run = 0;
    for c in text.chars() {
        run = if c.is_ascii_digit() { run + 1 } else { 0 };
        if run >= len {
            return true;
        }
    }
    false
}

#[test]
fn the_full_cpf_never_appears_in_the_label() {
    let mut infos: Vec<(String, CertInfo)> = vec![
        (
            "cpf".into(),
            icp(Some("12345678909"), Some("12345678000190")),
        ),
        ("cpf-only".into(), icp(Some("98765432100"), None)),
    ];
    for name in cert_names() {
        infos.push((name.clone(), info(&name)));
    }
    for (name, info) in infos {
        let Some(label) = display_document(&info) else {
            continue;
        };
        let text = format!("{label:?}");
        if let Some(cpf) = info.icp_brasil.as_ref().and_then(|i| i.cpf.as_deref()) {
            assert!(!text.contains(cpf), "{name}: {text}");
        }
        if matches!(label, DocumentLabel::Cpf { .. }) {
            assert!(!has_digit_run(&text, 7), "{name}: {text}");
        }
    }
}

#[test]
fn the_full_national_identifier_never_appears_in_the_label() {
    let text = format!("{:?}", display_document(&with_serial("IDCPT-12345123")));
    assert!(!text.contains("12345123"));
    assert!(!text.contains("IDCPT"));
}

#[test]
fn never_panics_on_odd_fields() {
    let long = "A".repeat(10_000);
    let odd = [
        "\u{0}",
        "•••••",
        "IDCPT-\u{1f4a9}\u{1f4a9}\u{1f4a9}",
        "IDCPT-é",
        "IDCPT-ééé",
        long.as_str(),
    ];
    for value in odd {
        let _ = display_document(&with_serial(value));
        let _ = display_document(&icp(Some(value), Some(value)));
    }
}
