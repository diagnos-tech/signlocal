//! IcpLevel and IcpBrasil.

use super::*;

#[test]
fn displays_known_levels_by_their_short_name() {
    let names = [
        (A1, "A1"),
        (A2, "A2"),
        (A3, "A3"),
        (A4, "A4"),
        (S1, "S1"),
        (S2, "S2"),
        (S3, "S3"),
        (T3, "T3"),
        (S4, "S4"),
        (T4, "T4"),
    ];
    for (level, name) in names {
        assert_eq!(level.to_string(), name);
    }
}

#[test]
fn displays_unknown_levels_with_their_arc() {
    assert_eq!(Other(999).to_string(), "ICP-Brasil (999)");
    assert_eq!(Other(0).to_string(), "ICP-Brasil (0)");
    assert_eq!(Other(5).to_string(), "ICP-Brasil (5)");
    assert_eq!(Other(u32::MAX).to_string(), "ICP-Brasil (4294967295)");
}

#[test]
fn level_is_copy_eq_and_hashable() {
    let level = A3;
    let copy = level;
    assert_eq!(level, copy);
    assert_ne!(A3, Other(3), "a hand-built Other(3) is not A3");
    assert_ne!(Other(1), Other(2));
    let all = [A1, A2, A3, A4, S1, S2, S3, S4, T3, T4, Other(7)];
    assert_eq!(all.into_iter().collect::<HashSet<_>>().len(), 11);
}

#[test]
fn icp_brasil_defaults_to_nothing_known() {
    assert_eq!(IcpBrasil::default(), expected(None, None, None, None));
}
