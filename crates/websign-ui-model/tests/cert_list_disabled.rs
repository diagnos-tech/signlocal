//! SPEC §1.5: listed-but-disabled rows ("Can't sign"), first reason wins.

mod common;

use common::*;
use websign_core::SignatureAlgorithm;
use websign_ui_model::certs::{CertCandidate, DisabledReason, PinMode, RowStatus, build_cert_list};

/// Lists `candidate` alone at [`now`] and returns its status.
fn status_alone(
    candidate: CertCandidate,
    context: websign_ui_model::certs::ListContext,
) -> RowStatus {
    let fingerprint = candidate.fingerprint;
    let list = build_cert_list(&[candidate], &context);
    assert!(list.hidden.is_empty(), "must be listed: {:?}", list.hidden);
    status(&list, fingerprint)
}

fn locked() -> PinMode {
    PinMode::App {
        length: None,
        count_low: false,
        final_try: false,
        locked: true,
    }
}

fn accepting(algorithms: &[SignatureAlgorithm]) -> websign_ui_model::certs::ListContext {
    let mut c = context();
    c.accepted = algorithms.to_vec();
    c
}

#[test]
fn a_valid_certificate_is_usable_and_lands_in_the_usable_group() {
    let list = build_cert_list(&[candidate(1, "Ana")], &context());
    assert_eq!(usable(&list), vec![fp(1)]);
    assert!(list.disabled.is_empty());
    assert_eq!(list.usable[0].status, RowStatus::Usable);
}

#[test]
fn a_removed_token_disables_its_row() {
    let mut c = candidate(1, "Ana");
    c.removed = true;
    assert_eq!(
        status_alone(c, context()),
        RowStatus::Disabled(DisabledReason::Removed)
    );
}

#[test]
fn expired_certificates_are_listed_as_disabled_not_hidden() {
    let c = with_info(candidate(1, "Ana"), |i| i.not_after = noon(2026, 5, 10));
    let list = build_cert_list(&[c], &context());
    assert!(list.usable.is_empty());
    assert_eq!(disabled(&list), vec![fp(1)]);
    assert_eq!(
        status(&list, fp(1)),
        RowStatus::Disabled(DisabledReason::Expired)
    );
}

#[test]
fn certificates_not_yet_valid_are_disabled() {
    let c = with_info(candidate(1, "Ana"), |i| i.not_before = noon(2026, 10, 1));
    assert_eq!(
        status_alone(c, context()),
        RowStatus::Disabled(DisabledReason::NotYetValid)
    );
}

#[test]
fn validity_is_inclusive_at_both_ends_of_the_period() {
    let at_end = with_info(candidate(1, "A"), |i| i.not_after = now());
    assert_eq!(status_alone(at_end, context()), RowStatus::Usable);
    let at_start = with_info(candidate(2, "B"), |i| i.not_before = now());
    assert_eq!(status_alone(at_start, context()), RowStatus::Usable);
    let past_end = with_info(candidate(3, "C"), |i| i.not_after = now() - 1);
    assert_eq!(
        status_alone(past_end, context()),
        RowStatus::Disabled(DisabledReason::Expired)
    );
    let before_start = with_info(candidate(4, "D"), |i| i.not_before = now() + 1);
    assert_eq!(
        status_alone(before_start, context()),
        RowStatus::Disabled(DisabledReason::NotYetValid)
    );
}

#[test]
fn a_locked_pin_disables_the_row_and_an_unlocked_app_pin_does_not() {
    let mut c = candidate(1, "Ana");
    c.pin = locked();
    assert_eq!(
        status_alone(c, context()),
        RowStatus::Disabled(DisabledReason::PinLocked)
    );

    let mut c = candidate(2, "Bia");
    c.pin = app_pin(Some((4, 16)));
    assert_eq!(status_alone(c, context()), RowStatus::Usable);
}

#[test]
fn no_algorithm_in_common_with_the_request_is_incompatible() {
    let c = candidate(1, "Ana"); // RSA PKCS#1 and PSS
    assert_eq!(
        status_alone(c, accepting(&[SignatureAlgorithm::Ecdsa])),
        RowStatus::Disabled(DisabledReason::Incompatible)
    );
}

#[test]
fn one_shared_algorithm_is_enough_and_an_empty_request_accepts_any() {
    let c = candidate(1, "Ana");
    let context = accepting(&[SignatureAlgorithm::Ecdsa, SignatureAlgorithm::RsaPss]);
    assert_eq!(status_alone(c.clone(), context), RowStatus::Usable);
    assert_eq!(status_alone(c, accepting(&[])), RowStatus::Usable);
}

#[test]
fn the_first_matching_disabled_reason_wins() {
    // Removed beats expired.
    let mut c = with_info(candidate(1, "A"), |i| i.not_after = noon(2026, 1, 1));
    c.removed = true;
    assert_eq!(
        status_alone(c, context()),
        RowStatus::Disabled(DisabledReason::Removed)
    );

    // Expired beats not-yet-valid is impossible; expired beats a locked PIN.
    let mut c = with_info(candidate(2, "B"), |i| i.not_after = noon(2026, 1, 1));
    c.pin = locked();
    assert_eq!(
        status_alone(c, context()),
        RowStatus::Disabled(DisabledReason::Expired)
    );

    // Not-yet-valid beats a locked PIN.
    let mut c = with_info(candidate(3, "C"), |i| i.not_before = noon(2026, 10, 1));
    c.pin = locked();
    assert_eq!(
        status_alone(c, context()),
        RowStatus::Disabled(DisabledReason::NotYetValid)
    );

    // A locked PIN beats incompatibility.
    let mut c = candidate(4, "D");
    c.pin = locked();
    assert_eq!(
        status_alone(c, accepting(&[SignatureAlgorithm::Ecdsa])),
        RowStatus::Disabled(DisabledReason::PinLocked)
    );
}

#[test]
fn disabled_rows_follow_the_usable_ordering_rules() {
    let expired = |n, name: &str, hardware| {
        let mut c = with_info(candidate(n, name), |i| i.not_after = noon(2026, 1, 1));
        c.hardware = hardware;
        c
    };
    let list = build_cert_list(
        &[
            expired(1, "Zeca", Some(false)),
            expired(2, "Bia", None),
            expired(3, "Ana", Some(true)),
            expired(4, "Ana", Some(true)),
        ],
        &context(),
    );
    // Hardware first, then name; equal names keep the input order? No rule:
    // only the two distinct names are pinned down.
    let order = disabled(&list);
    assert_eq!(order.len(), 4);
    assert_eq!(&order[2..], &[fp(2), fp(1)]);
    assert!(order[..2].contains(&fp(3)) && order[..2].contains(&fp(4)));
}
