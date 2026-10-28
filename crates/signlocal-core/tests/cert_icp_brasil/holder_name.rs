//! Holder name.

use super::*;

#[test]
fn strips_a_trailing_colon_and_digits_from_the_common_name() {
    let cases = [
        ("icp-cn-plain", "MARIA SILVA"),
        ("icp-cn-suffix", "MARIA SILVA"),
        ("icp-cn-nested-colon", "R2:D2"),
        ("icp-cn-accented", "JOSÉ D'ÁVILA"),
        ("icp-pj-a1", "EMPRESA TESTE LTDA"),
    ];
    for (name, holder) in cases {
        assert_eq!(icp(name).holder_name.as_deref(), Some(holder), "{name}");
    }
}

#[test]
fn keeps_a_suffix_that_is_not_only_digits() {
    let cases = [
        ("icp-cn-alpha-suffix", "MARIA SILVA:ABC"),
        ("icp-cn-mixed-suffix", "MARIA SILVA:123ABC"),
        ("icp-cn-space-suffix", "MARIA SILVA: 123"),
    ];
    for (name, holder) in cases {
        assert_eq!(icp(name).holder_name.as_deref(), Some(holder), "{name}");
    }
}

#[test]
fn holder_is_none_without_a_common_name() {
    let icp = icp("icp-no-cn");
    assert_eq!(icp.holder_name, None);
    assert_eq!(icp.level, Some(A1), "still ICP-Brasil through its policy");
}
