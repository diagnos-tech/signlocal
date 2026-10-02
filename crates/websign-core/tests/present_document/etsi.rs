//! ETSI EN 319 412-1 national identifiers.

use super::*;

#[test]
fn accepts_the_etsi_natural_person_types() {
    let cases = [
        ("IDCPT-12345123", "•••••123"),
        ("PNOPT-1234", "•••••234"),
        ("IDCES-123", "•••••123"),
        ("TINIT-ABC", "•••••ABC"),
        ("PNODE-12345678", "•••••678"),
        ("PASFR-09AB12345", "•••••345"),
        ("TAXBE-12345678901", "•••••901"),
        ("IDCPT-ABCDEFGH-1234", "•••••234"),
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
        "ZZZZZ-999",
        "VATPT-123456789",
        "NTRPT-123456789",
        "IDCP1-12345",
        "IDCpt-12345",
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
