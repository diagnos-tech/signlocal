use websign_protocol::types::{EidasProfile, EidasType, KeyStorage};

use super::{icp_info, info_with};
use crate::CertInfo;
use crate::present::wire::certificate_profile;
use crate::testkit::*;

#[test]
fn icp_brasil_level_is_named() {
    let info = icp_info("ANA", None, None);
    let profile = certificate_profile(&info, Some(true));
    assert_eq!(profile.icp_brasil.as_deref(), Some("A3"));
    assert_eq!(profile.eidas, None);
    assert_eq!(profile.key_storage, KeyStorage::Hardware);
}

#[test]
fn unknown_icp_level_and_plain_certificates() {
    let other = TestCert::new().extension(policies(&["2.16.76.1.2.999.1"]));
    let info = CertInfo::from_der(&other.build()).expect("parses");
    assert_eq!(
        certificate_profile(&info, Some(false))
            .icp_brasil
            .as_deref(),
        Some("ICP-Brasil")
    );
    let by_name = TestCert::new().extension(san_other_names(&[("2.16.76.1.3.2", tlv(UTF8, b"N"))]));
    let info = CertInfo::from_der(&by_name.build()).expect("parses");
    assert_eq!(
        certificate_profile(&info, None).icp_brasil.as_deref(),
        Some("ICP-Brasil")
    );
    let plain = info_with(&[rdn(CN, UTF8, b"X")]);
    let profile = certificate_profile(&plain, None);
    assert_eq!(profile.icp_brasil, None);
    assert_eq!(profile.key_storage, KeyStorage::Unknown);
}

#[test]
fn eidas_statements_are_mapped() {
    let types = tlv(
        0x30,
        &[oid("0.4.0.1862.1.6.1"), oid("0.4.0.1862.1.6.3")].concat(),
    );
    let cert = TestCert::new().extension(qc_statements(&[
        ("0.4.0.1862.1.1", None),
        ("0.4.0.1862.1.6", Some(types)),
    ]));
    let info = CertInfo::from_der(&cert.build()).expect("parses");
    assert_eq!(
        certificate_profile(&info, Some(false)).eidas,
        Some(EidasProfile {
            qualified: true,
            qscd: false,
            types: vec![EidasType::Esign, EidasType::Web],
        })
    );
}
