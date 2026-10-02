//! SPEC §1.1: the lines of a row, filled from the candidate and the core
//! presentation rules.

mod common;

use common::*;
use websign_ui_model::certs::build_cert_list;

#[test]
fn row_lines_are_filled_from_the_candidate() {
    let mut c = with_info(candidate(1, "Ana Beatriz Souza"), |i| {
        i.issuer.common_name = Some("AC Soluti".to_owned());
        i.not_after = noon(2026, 10, 22);
    });
    c.hardware = Some(false);
    let list = build_cert_list(&[c], &context());
    let row = &list.usable[0];
    assert_eq!(row.name, "Ana Beatriz Souza");
    assert_eq!(row.issuer, "AC Soluti");
    assert_eq!(row.document, None);
    assert_eq!(row.location.place, websign_ui_model::certs::Place::Computer);
    assert_eq!(
        row.validity,
        websign_ui_model::certs::ValidityLabel::ExpiresInDays(23)
    );
}

#[test]
fn the_issuer_falls_back_to_the_organization_then_to_empty() {
    let by_org = with_info(candidate(1, "A"), |i| {
        i.issuer.common_name = None;
        i.issuer.organization = Some("Certisign".to_owned());
    });
    let neither = with_info(candidate(2, "B"), |i| {
        i.issuer.common_name = None;
        i.issuer.organization = None;
    });
    let list = build_cert_list(&[by_org, neither], &context());
    assert_eq!(row(&list, fp(1)).issuer, "Certisign");
    assert_eq!(row(&list, fp(2)).issuer, "");
}

#[test]
fn the_row_document_and_badge_come_from_the_core_rules() {
    use websign_core::{IcpBrasil, IcpLevel, present::document::display_document};
    let c = with_info(candidate(1, "Ana"), |i| {
        i.icp_brasil = Some(IcpBrasil {
            level: Some(IcpLevel::A3),
            holder_name: Some("Ana".to_owned()),
            cpf: Some("12345678901".to_owned()),
            cnpj: None,
        });
    });
    let expected = display_document(c.info.as_ref().unwrap());
    let list = build_cert_list(&[c], &context());
    assert!(expected.is_some());
    assert_eq!(list.usable[0].document, expected);
    assert_eq!(
        list.usable[0].badge,
        websign_ui_model::certs::Badge::IcpBrasil {
            class: "A3".to_owned()
        }
    );
}
