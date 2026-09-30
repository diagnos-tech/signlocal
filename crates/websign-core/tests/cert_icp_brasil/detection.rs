//! What makes a certificate ICP-Brasil.

use super::*;

#[test]
fn recognizes_the_policy_alone() {
    assert_eq!(
        info("icp-policy-only").icp_brasil,
        Some(expected(Some(A1), Some("POLICY ONLY"), None, None))
    );
}

#[test]
fn recognizes_a_person_other_name_alone() {
    assert_eq!(
        info("icp-san-only").icp_brasil,
        Some(expected(None, Some("SAN ONLY"), Some(CPF), None))
    );
}

#[test]
fn any_other_name_under_2_16_76_1_3_marks_the_certificate() {
    // 2.16.76.1.3.2 (responsible's name) carries no CPF or CNPJ, but is ICP-Brasil.
    assert_eq!(
        info("icp-san-other-arc").icp_brasil,
        Some(expected(None, Some("OTHER ARC"), None, None))
    );
}

#[test]
fn oids_that_only_share_a_digit_prefix_do_not_count() {
    // Policies 2.16.76.1.20.3 and 2.16.760.1.2.3.1, otherName 2.16.76.1.30.1.
    let info = info("icp-lookalike");
    assert_eq!(info.policies, ["2.16.76.1.20.3", "2.16.760.1.2.3.1"]);
    assert_eq!(info.icp_brasil, None);
}

#[test]
fn other_general_names_and_foreign_other_names_do_not_count() {
    // e-mail, DNS and a Microsoft UPN otherName.
    assert_eq!(info("non-icp-upn").icp_brasil, None);
}
