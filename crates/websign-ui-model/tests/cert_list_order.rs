//! SPEC §1.5: order of usable rows and the initial selection.

mod common;

use common::*;
use websign_ui_model::certs::{CertCandidate, ListContext, build_cert_list};

fn hw(n: u8, name: &str, hardware: Option<bool>) -> CertCandidate {
    let mut c = candidate(n, name);
    c.hardware = hardware;
    c
}

fn order(candidates: &[CertCandidate], context: &ListContext) -> Vec<u8> {
    let list = build_cert_list(candidates, context);
    usable(&list).iter().map(|f| f.as_bytes()[0]).collect()
}

#[test]
fn never_used_certificates_put_hardware_before_unknown_before_software() {
    let candidates = [
        hw(1, "Ana", Some(false)),
        hw(2, "Ana", None),
        hw(3, "Ana", Some(true)),
    ];
    assert_eq!(order(&candidates, &context()), vec![3, 2, 1]);
}

#[test]
fn never_used_certificates_of_equal_kind_sort_by_name_ignoring_case_and_accents() {
    let candidates = [
        hw(1, "Carla", Some(true)),
        hw(2, "bruno", Some(true)),
        hw(3, "Álvaro", Some(true)),
        hw(4, "alberto", Some(true)),
    ];
    // alberto < Álvaro (accent ignored: "alberto" < "alvaro") < bruno < Carla
    assert_eq!(order(&candidates, &context()), vec![4, 3, 2, 1]);
}

#[test]
fn never_used_certificates_of_equal_name_put_the_longer_validity_first() {
    let short = with_info(hw(1, "Ana", Some(true)), |i| i.not_after = noon(2027, 1, 1));
    let long = with_info(hw(2, "Ana", Some(true)), |i| i.not_after = noon(2029, 1, 1));
    assert_eq!(order(&[short, long], &context()), vec![2, 1]);
}

#[test]
fn hardware_outranks_name_and_name_outranks_validity() {
    let candidates = [
        with_info(hw(1, "Zoe", Some(true)), |i| i.not_after = noon(2027, 1, 1)),
        with_info(hw(2, "Ana", Some(false)), |i| {
            i.not_after = noon(2030, 1, 1)
        }),
        with_info(hw(3, "Bia", Some(false)), |i| {
            i.not_after = noon(2031, 1, 1)
        }),
        with_info(hw(4, "Bia", Some(false)), |i| {
            i.not_after = noon(2035, 1, 1)
        }),
    ];
    assert_eq!(order(&candidates, &context()), vec![1, 2, 4, 3]);
}

#[test]
fn recently_used_certificates_follow_their_position_in_recent_anywhere() {
    let candidates = [
        hw(1, "Ana", Some(false)),
        hw(2, "Bia", Some(false)),
        hw(3, "Carla", Some(false)),
        hw(4, "Duda", Some(true)),
    ];
    let mut context = context();
    context.recent_anywhere = vec![fp(3), fp(1)];
    // Used ones first in recency order, then never used (hardware first).
    assert_eq!(order(&candidates, &context), vec![3, 1, 4, 2]);
}

#[test]
fn recently_used_software_still_outranks_never_used_hardware() {
    let candidates = [hw(1, "Ana", Some(true)), hw(2, "Zeca", Some(false))];
    let mut context = context();
    context.recent_anywhere = vec![fp(2)];
    assert_eq!(order(&candidates, &context), vec![2, 1]);
}

#[test]
fn recent_fingerprints_of_unknown_certificates_are_ignored() {
    let candidates = [hw(1, "Ana", Some(false)), hw(2, "Bia", Some(false))];
    let mut context = context();
    context.recent_anywhere = vec![fp(99), fp(2), fp(98)];
    assert_eq!(order(&candidates, &context), vec![2, 1]);
}

#[test]
fn the_certificate_last_used_here_comes_first() {
    let candidates = [
        hw(1, "Ana", Some(true)),
        hw(2, "Bia", Some(false)),
        hw(3, "Carla", Some(false)),
    ];
    let mut context = context();
    context.last_used_here = Some(fp(3));
    context.recent_anywhere = vec![fp(2), fp(3)];
    assert_eq!(order(&candidates, &context), vec![3, 2, 1]);
}

#[test]
fn a_disabled_last_used_certificate_is_neither_first_nor_selected() {
    let expired = with_info(hw(3, "Carla", Some(false)), |i| {
        i.not_after = noon(2026, 1, 1)
    });
    let candidates = [
        hw(1, "Ana", Some(false)),
        hw(2, "Bia", Some(false)),
        expired,
    ];
    let mut context = context();
    context.last_used_here = Some(fp(3));
    let list = build_cert_list(&candidates, &context);
    assert_eq!(usable(&list), vec![fp(1), fp(2)]);
    assert_eq!(list.selected, Some(fp(1)));
}

#[test]
fn a_last_used_certificate_that_is_gone_changes_nothing() {
    let candidates = [hw(1, "Ana", Some(false)), hw(2, "Bia", Some(false))];
    let mut context = context();
    context.last_used_here = Some(fp(77));
    let list = build_cert_list(&candidates, &context);
    assert_eq!(usable(&list), vec![fp(1), fp(2)]);
    assert_eq!(list.selected, Some(fp(1)));
}

#[test]
fn selection_is_the_first_usable_row_by_default() {
    let candidates = [hw(1, "Zoe", Some(false)), hw(2, "Ana", Some(true))];
    let list = build_cert_list(&candidates, &context());
    assert_eq!(list.selected, Some(fp(2)));
}

#[test]
fn selection_prefers_last_used_here() {
    let candidates = [hw(1, "Ana", Some(true)), hw(2, "Bia", Some(false))];
    let mut context = context();
    context.last_used_here = Some(fp(2));
    assert_eq!(build_cert_list(&candidates, &context).selected, Some(fp(2)));
}

#[test]
fn a_usable_requested_certificate_wins_the_selection_but_not_the_order() {
    let candidates = [
        hw(1, "Ana", Some(true)),
        hw(2, "Bia", Some(false)),
        hw(3, "Carla", Some(false)),
    ];
    let mut context = context();
    context.last_used_here = Some(fp(2));
    context.requested = Some(fp(3));
    let list = build_cert_list(&candidates, &context);
    assert_eq!(list.selected, Some(fp(3)));
    assert_eq!(usable(&list), vec![fp(2), fp(1), fp(3)]);
}

#[test]
fn an_unusable_requested_certificate_falls_back_to_last_used_here() {
    let expired = with_info(hw(3, "Carla", Some(false)), |i| {
        i.not_after = noon(2026, 1, 1)
    });
    let candidates = [hw(1, "Ana", Some(true)), hw(2, "Bia", Some(false)), expired];
    let mut context = context();
    context.last_used_here = Some(fp(2));
    context.requested = Some(fp(3));
    assert_eq!(build_cert_list(&candidates, &context).selected, Some(fp(2)));
}

#[test]
fn a_hidden_or_unknown_requested_certificate_falls_back_to_the_first_usable() {
    let mut no_key = hw(3, "Carla", Some(false));
    no_key.has_private_key = false;
    let candidates = [hw(1, "Ana", Some(true)), no_key];
    for requested in [fp(3), fp(50)] {
        let mut context = context();
        context.requested = Some(requested);
        assert_eq!(build_cert_list(&candidates, &context).selected, Some(fp(1)));
    }
}

#[test]
fn nothing_usable_means_no_selection() {
    let expired = with_info(candidate(1, "Ana"), |i| i.not_after = noon(2026, 1, 1));
    let list = build_cert_list(&[expired], &context());
    assert_eq!(list.selected, None);
    let empty = build_cert_list(&[], &context());
    assert_eq!(empty.selected, None);
    assert!(empty.usable.is_empty() && empty.disabled.is_empty() && empty.hidden.is_empty());
}
