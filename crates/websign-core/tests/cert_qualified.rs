//! SPEC §6.4: eIDAS qcStatements (`Qualified`, `QcType`).
//!
//! Statement OIDs are 0.4.0.1862.1.<n>; `tests/fixtures/gen/certs-qualified.sh`
//! assembles each certificate's list and the README tabulates them.

mod common;

use std::collections::HashSet;

use common::{cert, info};
use websign_core::{CertError, CertInfo, QcType, Qualified};

use QcType::{ESeal, ESign, Web};

fn qualified(name: &str) -> Qualified {
    info(name)
        .qualified
        .unwrap_or_else(|| panic!("{name} should carry qcStatements"))
}

fn expected(compliance: bool, sscd: bool, types: &[QcType]) -> Qualified {
    Qualified {
        compliance,
        sscd,
        types: types.to_vec(),
    }
}

#[test]
fn reads_a_qualified_signature_certificate_on_a_qscd() {
    // QcCompliance + QcSSCD + QcType esign, surrounded by QcLimitValue,
    // QcRetentionPeriod, QcPDS and one private statement, none of which are read.
    assert_eq!(qualified("qc-esign-sscd"), expected(true, true, &[ESign]));
}

#[test]
fn reads_a_seal_certificate_without_a_qscd() {
    assert_eq!(qualified("qc-eseal"), expected(true, false, &[ESeal]));
}

#[test]
fn reads_a_web_certificate_that_is_not_marked_compliant() {
    assert_eq!(qualified("qc-web"), expected(false, false, &[Web]));
}

#[test]
fn reads_the_qscd_statement_on_its_own() {
    assert_eq!(qualified("qc-sscd-only"), expected(false, true, &[]));
}

#[test]
fn keeps_qc_types_in_certificate_order_and_skips_unknown_ones() {
    // The certificate lists 6.3, 6.9 (unknown), 6.1, 6.2.
    assert_eq!(qualified("qc-type-order").types, [Web, ESign, ESeal]);
}

#[test]
fn an_empty_qc_type_list_gives_no_types() {
    assert_eq!(qualified("qc-type-empty-info"), expected(false, false, &[]));
}

#[test]
fn an_empty_statement_list_is_still_a_qualified_extension() {
    // `Some` whenever the extension exists, whatever it holds.
    assert_eq!(qualified("qc-empty"), Qualified::default());
}

#[test]
fn statements_of_unknown_type_are_ignored() {
    assert_eq!(qualified("qc-unknown-only"), Qualified::default());
}

#[test]
fn certificates_without_the_extension_are_not_qualified() {
    for name in [
        "rsa2048",
        "p256",
        "ca",
        "icp-pf-a3",
        "icp-pj-a1",
        "v1",
        "bare",
        "non-icp-upn",
    ] {
        assert_eq!(info(name).qualified, None, "{name}");
    }
}

#[test]
fn qualified_certificates_are_not_taken_for_icp_brasil() {
    for name in ["qc-esign-sscd", "qc-eseal", "qc-web", "qc-empty"] {
        assert_eq!(info(name).icp_brasil, None, "{name}");
    }
}

#[test]
fn qualified_data_does_not_change_the_rest_of_the_summary() {
    let info = info("qc-esign-sscd");
    assert!(info.can_sign(), "nonRepudiation is set");
    assert_eq!(info.subject.country.as_deref(), Some("PT"));
    assert_eq!(
        info.subject.common_name.as_deref(),
        Some("QC fixture qc-esign-sscd")
    );
    assert!(info.policies.is_empty());
}

#[test]
fn rejects_a_qc_statements_extension_that_is_not_a_sequence() {
    assert!(matches!(
        CertInfo::from_der(&cert("bad-ext-qc-statements")),
        Err(CertError::Malformed(_))
    ));
}

#[test]
fn rejects_a_qc_statements_sequence_holding_something_else() {
    // SEQUENCE { INTEGER 5 } where SEQUENCE OF SEQUENCE is required.
    assert!(matches!(
        CertInfo::from_der(&cert("bad-qc-statement-element")),
        Err(CertError::Malformed(_))
    ));
}

#[test]
fn defaults_to_nothing_declared() {
    let empty = Qualified::default();
    assert!(!empty.compliance);
    assert!(!empty.sscd);
    assert!(empty.types.is_empty());
}

#[test]
fn qc_type_is_copy_eq_and_hashable() {
    let kind = ESign;
    let copy = kind;
    assert_eq!(kind, copy);
    assert_ne!(ESign, ESeal);
    assert_ne!(ESeal, Web);
    assert_eq!(
        [ESign, ESeal, Web]
            .into_iter()
            .collect::<HashSet<_>>()
            .len(),
        3
    );
}
