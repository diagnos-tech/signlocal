//! The ICP-Brasil class.

use super::*;

#[test]
fn a_certificate_that_is_not_icp_brasil_has_no_icp_profile() {
    let profile = certificate_profile(&blank_info(), None);
    assert_eq!(profile.icp_brasil, None);
}

#[test]
fn names_the_icp_brasil_class() {
    let levels = [
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
    for (level, text) in levels {
        let profile = certificate_profile(&icp_info(Some(level)), None);
        assert_eq!(profile.icp_brasil.as_deref(), Some(text), "{level}");
    }
}

#[test]
fn an_unknown_or_missing_class_is_plain_icp_brasil() {
    for level in [
        None,
        Some(IcpLevel::Other(0)),
        Some(IcpLevel::Other(999)),
        Some(IcpLevel::Other(u32::MAX)),
    ] {
        let profile = certificate_profile(&icp_info(level), None);
        assert_eq!(
            profile.icp_brasil.as_deref(),
            Some("ICP-Brasil"),
            "{level:?}"
        );
    }
}

#[test]
fn reads_the_class_from_real_certificates() {
    let cases = [
        ("icp-pf-a3", Some("A3")),
        ("icp-pj-a1", Some("A1")),
        ("icp-level-303", Some("T3")),
        ("icp-level-102", Some("S2")),
        ("icp-level-999", Some("ICP-Brasil")),
        ("icp-level-huge", Some("ICP-Brasil")),
        ("icp-san-only", Some("ICP-Brasil")),
        ("icp-lookalike", None),
        ("p256", None),
    ];
    for (name, expected) in cases {
        let profile = certificate_profile(&info(name), None);
        assert_eq!(profile.icp_brasil.as_deref(), expected, "{name}");
    }
}
