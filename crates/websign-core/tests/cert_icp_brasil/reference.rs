//! The reference certificates.

use super::*;

#[test]
fn reads_a_natural_person_a3_certificate() {
    let info = info("icp-pf-a3");
    assert_eq!(info.icp_brasil, Some(person(Some(CPF))));
    assert_eq!(info.policies, ["2.16.76.1.2.3.1"]);
    assert_eq!(
        info.subject.common_name.as_deref(),
        Some("ANA BEATRIZ SOUZA:12345678901")
    );
}

#[test]
fn reads_a_company_a1_certificate() {
    let info = info("icp-pj-a1");
    assert_eq!(
        info.icp_brasil,
        Some(company(Some(CNPJ), Some(RESPONSIBLE_CPF)))
    );
    assert_eq!(info.policies, ["2.16.76.1.2.1.1"]);
}

#[test]
fn certificates_outside_icp_brasil_have_no_icp_data() {
    for name in [
        "rsa2048",
        "p256",
        "ca",
        "qc-esign-sscd",
        "non-icp-upn",
        "v1",
        "dn-utf8",
    ] {
        assert_eq!(info(name).icp_brasil, None, "{name}");
    }
}
