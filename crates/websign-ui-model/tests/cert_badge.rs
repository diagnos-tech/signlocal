//! SPEC §1.2: the single badge of a row, first matching rule wins.

mod common;

use common::info;
use websign_core::{CertInfo, IcpBrasil, IcpLevel, Qualified};
use websign_ui_model::certs::{Badge, badge};

fn issuer(mut info: CertInfo, cn: Option<&str>, org: Option<&str>) -> CertInfo {
    info.issuer.common_name = cn.map(str::to_owned);
    info.issuer.organization = org.map(str::to_owned);
    info
}

fn icp(level: Option<IcpLevel>) -> Option<IcpBrasil> {
    Some(IcpBrasil {
        level,
        ..IcpBrasil::default()
    })
}

fn qualified(compliance: bool, sscd: bool) -> Option<Qualified> {
    Some(Qualified {
        compliance,
        sscd,
        types: Vec::new(),
    })
}

#[test]
fn portuguese_citizen_card_by_issuer_common_name_case_insensitively() {
    for cn in [
        "EC de Assinatura Digital Qualificada do Cartão de Cidadão 003",
        "EC DE ASSINATURA DIGITAL QUALIFICADA DO CARTÃO DE CIDADÃO 003",
        "cartão de cidadão",
    ] {
        let info = issuer(info(1, "Joao"), Some(cn), None);
        assert_eq!(badge(&info), Badge::PtCitizenCard, "{cn}");
    }
}

#[test]
fn spanish_dnie_by_issuer_organization_ignoring_case_and_accents() {
    for org in [
        "DIRECCION GENERAL DE LA POLICIA",
        "Direccion General de la Policia",
        "DIRECCIÓN GENERAL DE LA POLICÍA",
    ] {
        let info = issuer(info(1, "Juan"), Some("AC DNIE 004"), Some(org));
        assert_eq!(badge(&info), Badge::EsDnie, "{org}");
    }
}

#[test]
fn a_longer_organization_is_not_the_dnie() {
    // The organization must *equal* the name, not merely contain it.
    let info = issuer(
        info(1, "Juan"),
        None,
        Some("DIRECCION GENERAL DE LA POLICIA NACIONAL"),
    );
    assert_eq!(badge(&info), Badge::Generic);
}

#[test]
fn country_cards_win_over_icp_brasil_and_qualified() {
    let mut pt = issuer(info(1, "A"), Some("Cartão de Cidadão"), None);
    pt.icp_brasil = icp(Some(IcpLevel::A3));
    pt.qualified = qualified(true, true);
    assert_eq!(badge(&pt), Badge::PtCitizenCard);

    let mut es = issuer(info(2, "B"), None, Some("DIRECCION GENERAL DE LA POLICIA"));
    es.icp_brasil = icp(Some(IcpLevel::A1));
    es.qualified = qualified(true, true);
    assert_eq!(badge(&es), Badge::EsDnie);
}

#[test]
fn recognized_icp_brasil_levels_carry_their_class() {
    let table = [
        (IcpLevel::A1, "A1"),
        (IcpLevel::A2, "A2"),
        (IcpLevel::A3, "A3"),
        (IcpLevel::A4, "A4"),
        (IcpLevel::S1, "S1"),
        (IcpLevel::S2, "S2"),
        (IcpLevel::S3, "S3"),
        (IcpLevel::S4, "S4"),
        (IcpLevel::T3, "T3"),
        (IcpLevel::T4, "T4"),
    ];
    for (level, class) in table {
        let mut info = info(1, "Ana");
        info.icp_brasil = icp(Some(level));
        assert_eq!(
            badge(&info),
            Badge::IcpBrasil {
                class: class.to_owned()
            }
        );
    }
}

#[test]
fn icp_brasil_without_a_recognized_level_is_plain() {
    for level in [None, Some(IcpLevel::Other(99))] {
        let mut info = info(1, "Ana");
        info.icp_brasil = icp(level);
        assert_eq!(badge(&info), Badge::IcpBrasilPlain);
    }
}

#[test]
fn icp_brasil_wins_over_qualified() {
    let mut info = info(1, "Ana");
    info.icp_brasil = icp(None);
    info.qualified = qualified(true, true);
    assert_eq!(badge(&info), Badge::IcpBrasilPlain);
}

#[test]
fn qualified_with_sscd_is_eidas_qualified() {
    let mut info = info(1, "Ana");
    info.qualified = qualified(true, true);
    assert_eq!(badge(&info), Badge::EidasQualified);
}

#[test]
fn qualified_without_sscd_is_eidas() {
    let mut info = info(1, "Ana");
    info.qualified = qualified(true, false);
    assert_eq!(badge(&info), Badge::Eidas);
}

#[test]
fn sscd_without_compliance_is_generic() {
    // Rules 5 and 6 both require `compliance`.
    let mut info = info(1, "Ana");
    info.qualified = qualified(false, true);
    assert_eq!(badge(&info), Badge::Generic);
}

#[test]
fn everything_else_is_generic() {
    assert_eq!(badge(&info(1, "Ana")), Badge::Generic);
    let mut info = info(2, "Ana");
    info.qualified = qualified(false, false);
    assert_eq!(badge(&info), Badge::Generic);
}
