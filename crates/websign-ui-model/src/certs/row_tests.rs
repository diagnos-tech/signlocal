//! Badge, location and filter rules.

use websign_core::present::document::DocumentLabel;
use websign_core::{IcpBrasil, IcpLevel, Qualified};

use super::*;
use crate::fixtures::{candidate, context, info};

fn with_level(level: Option<IcpLevel>) -> websign_core::CertInfo {
    let mut cert = info(1, "X");
    cert.icp_brasil = Some(IcpBrasil {
        level,
        ..Default::default()
    });
    cert
}

#[test]
fn badge_rules_in_order() {
    let mut cert = with_level(Some(IcpLevel::A3));
    assert_eq!(badge(&cert), Badge::IcpBrasil { class: "A3".into() });
    cert.issuer.organization = Some("Dirección General de la Policía".into());
    assert_eq!(badge(&cert), Badge::EsDnie);
    cert.issuer.common_name = Some("EC de Cartão de Cidadão 012".into());
    assert_eq!(badge(&cert), Badge::PtCitizenCard);
    assert_eq!(
        badge(&with_level(Some(IcpLevel::Other(9)))),
        Badge::IcpBrasilPlain
    );
    assert_eq!(badge(&with_level(None)), Badge::IcpBrasilPlain);
}

#[test]
fn badge_eidas_and_generic() {
    let mut cert = info(1, "X");
    cert.icp_brasil = None;
    assert_eq!(badge(&cert), Badge::Generic);
    let qualified = |compliance, sscd| Qualified {
        compliance,
        sscd,
        types: Vec::new(),
    };
    cert.qualified = Some(qualified(true, true));
    assert_eq!(badge(&cert), Badge::EidasQualified);
    cert.qualified = Some(qualified(true, false));
    assert_eq!(badge(&cert), Badge::Eidas);
    cert.qualified = Some(qualified(false, true));
    assert_eq!(badge(&cert), Badge::Generic);
}

#[test]
fn locations() {
    let mut c = candidate(1, "X");
    c.hardware = Some(false);
    assert_eq!(location(&c).place, Place::Computer);
    c.hardware = Some(true);
    assert_eq!(location(&c).place, Place::UnknownHardware);
    c.device = Some(DeviceLabel::Token {
        name: "SafeNet eToken 5110".into(),
    });
    c.source = KeySource::Driver { path: "p".into() };
    let found = location(&c);
    assert_eq!(
        found.place,
        Place::Token {
            name: "SafeNet eToken 5110".into()
        }
    );
    assert!(found.via_driver);
    c.device = Some(DeviceLabel::CardInReader { reader: "R".into() });
    assert_eq!(location(&c).place, Place::CardInReader);
    c.device = Some(DeviceLabel::Unknown);
    assert_eq!(location(&c).place, Place::UnknownHardware);
    c.hardware = Some(false);
    assert_eq!(location(&c).place, Place::Computer);
}

fn row(name: &str, issuer: &str, document: Option<DocumentLabel>) -> CertRow {
    let list = build_cert_list(&[candidate(1, name)], &context());
    let mut row = list.usable[0].clone();
    row.issuer = issuer.to_owned();
    row.document = document;
    row
}

#[test]
fn filter_by_text_ignores_case_and_accents() {
    let row = row("João Álvaro", "AC Certisign", None);
    assert!(matches_filter(&row, ""));
    assert!(matches_filter(&row, "  "));
    assert!(matches_filter(&row, "joao alvaro"));
    assert!(matches_filter(&row, "CERTISIGN"));
    assert!(!matches_filter(&row, "serasa"));
}

#[test]
fn filter_by_document_digits() {
    let cpf = row(
        "Ana",
        "AC",
        Some(DocumentLabel::Cpf {
            masked: "•••.456.789-••".into(),
            visible: "456 789".into(),
        }),
    );
    assert!(matches_filter(&cpf, "456.789"));
    assert!(matches_filter(&cpf, "456789"));
    assert!(!matches_filter(&cpf, "123"));
    let cnpj = row(
        "Clinica",
        "AC",
        Some(DocumentLabel::Cnpj {
            formatted: "12.345.678/0001-90".into(),
        }),
    );
    assert!(matches_filter(&cnpj, "12.345.678/0001"));
    let national = row(
        "Rui",
        "AC",
        Some(DocumentLabel::National {
            masked: "•••••123".into(),
        }),
    );
    assert!(matches_filter(&national, "123"));
    assert!(!matches_filter(&national, "999"));
}
