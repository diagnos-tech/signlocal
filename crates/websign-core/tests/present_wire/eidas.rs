//! The eIDAS profile.

use super::*;

#[test]
fn a_certificate_without_qc_statements_has_no_eidas_profile() {
    assert_eq!(certificate_profile(&blank_info(), None).eidas, None);
}

#[test]
fn maps_compliance_sscd_and_types() {
    let info = qualified_info(true, true, vec![QcType::ESign]);
    assert_eq!(
        certificate_profile(&info, None).eidas,
        eidas(true, true, vec![EidasType::Esign])
    );
}

#[test]
fn compliance_is_qualified_and_sscd_is_qscd() {
    let only_compliance = qualified_info(true, false, vec![]);
    assert_eq!(
        certificate_profile(&only_compliance, None).eidas,
        eidas(true, false, vec![])
    );
    let only_sscd = qualified_info(false, true, vec![]);
    assert_eq!(
        certificate_profile(&only_sscd, None).eidas,
        eidas(false, true, vec![])
    );
}

#[test]
fn present_but_empty_qc_statements_still_give_a_profile() {
    let info = qualified_info(false, false, vec![]);
    assert_eq!(
        certificate_profile(&info, None).eidas,
        eidas(false, false, vec![])
    );
}

#[test]
fn maps_every_type_keeping_order_and_repeats() {
    let info = qualified_info(
        true,
        false,
        vec![QcType::Web, QcType::ESign, QcType::ESeal, QcType::Web],
    );
    assert_eq!(
        certificate_profile(&info, None).eidas,
        eidas(
            true,
            false,
            vec![
                EidasType::Web,
                EidasType::Esign,
                EidasType::Eseal,
                EidasType::Web
            ]
        )
    );
}

#[test]
fn reads_the_profile_from_real_certificates() {
    let cases = [
        ("qc-esign-sscd", eidas(true, true, vec![EidasType::Esign])),
        ("qc-eseal", eidas(true, false, vec![EidasType::Eseal])),
        ("qc-web", eidas(false, false, vec![EidasType::Web])),
        (
            "qc-type-order",
            eidas(
                false,
                false,
                vec![EidasType::Web, EidasType::Esign, EidasType::Eseal],
            ),
        ),
        ("qc-sscd-only", eidas(false, true, vec![])),
        ("qc-unknown-only", eidas(false, false, vec![])),
        ("qc-empty", eidas(false, false, vec![])),
        ("p256", None),
        ("icp-pf-a3", None),
    ];
    for (name, expected) in cases {
        assert_eq!(
            certificate_profile(&info(name), None).eidas,
            expected,
            "{name}"
        );
    }
}
