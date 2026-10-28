//! Level.

use super::*;

#[test]
fn maps_policy_arcs_to_levels() {
    let table: [(u32, IcpLevel); 18] = [
        (0, Other(0)),
        (1, A1),
        (2, A2),
        (3, A3),
        (4, A4),
        (5, Other(5)),
        (100, Other(100)),
        (101, S1),
        (102, S2),
        (103, S3),
        (104, S4),
        (105, Other(105)),
        (302, Other(302)),
        (303, T3),
        (304, T4),
        (305, Other(305)),
        (999, Other(999)),
        (u32::MAX, Other(u32::MAX)),
    ];
    for (arc, level) in table {
        let icp = icp(&format!("icp-level-{arc}"));
        assert_eq!(icp.level, Some(level), "policy 2.16.76.1.2.{arc}.1");
        assert_eq!(
            icp.holder_name,
            Some(format!("LEVEL {arc}")),
            "holder for {arc}"
        );
    }
}

#[test]
fn uses_the_first_icp_policy_and_skips_foreign_ones_before_it() {
    let info = info("icp-multi-policy");
    assert_eq!(
        info.policies,
        ["1.2.3.4", "2.16.76.1.2.3.4", "2.16.76.1.2.1.2"]
    );
    assert_eq!(info.icp_brasil.expect("icp").level, Some(A3));
}

#[test]
fn the_first_icp_policy_wins_even_when_its_arc_is_unknown() {
    let info = info("icp-multi-policy-other-first");
    assert_eq!(info.policies, ["2.16.76.1.2.999.1", "2.16.76.1.2.3.1"]);
    assert_eq!(info.icp_brasil.expect("icp").level, Some(Other(999)));
}

#[test]
fn reads_the_level_from_a_policy_without_further_arcs() {
    // A bare 2.16.76.1.2.3 still starts with the ICP prefix, so it reads n = 3.
    assert_eq!(icp("icp-level-no-subarc").level, Some(A3));
}

#[test]
fn keeps_a_policy_arc_beyond_u32_but_leaves_the_level_unknown() {
    // `Other(n)` holds a u32, so 4294967296 has no level; the certificate
    // is still ICP-Brasil and its policy is listed verbatim.
    let info = info("icp-level-huge");
    assert_eq!(info.policies, ["2.16.76.1.2.4294967296.1"]);
    let icp = info.icp_brasil.expect("ICP-Brasil by its policy");
    assert_eq!(icp.level, None);
    assert_eq!(icp.holder_name.as_deref(), Some("LEVEL HUGE"));
}

#[test]
fn level_is_none_without_an_icp_policy() {
    assert_eq!(icp("icp-san-only").level, None);
    assert_eq!(icp("icp-san-other-arc").level, None);
}
