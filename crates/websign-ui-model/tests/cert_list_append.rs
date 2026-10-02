//! SPEC §1.6: merging a fresh listing into an open window's list.

mod common;

use common::*;
use websign_ui_model::certs::{
    CertCandidate, CertList, DisabledReason, HiddenReason, RowStatus, build_cert_list,
};

fn ana() -> CertCandidate {
    candidate(1, "Ana")
}
fn bia() -> CertCandidate {
    candidate(2, "Bia")
}
fn carla() -> CertCandidate {
    candidate(3, "Carla")
}

fn list_of(candidates: &[CertCandidate]) -> CertList {
    build_cert_list(candidates, &context())
}

#[test]
fn a_new_token_is_appended_at_the_end_and_the_selection_stays() {
    let mut list = list_of(&[ana(), bia()]);
    assert_eq!(list.selected, Some(fp(1)));

    // The newcomer would sort first by every rule.
    let mut newcomer = candidate(4, "Aaron");
    newcomer.hardware = Some(true);
    let mut context = context();
    context.last_used_here = Some(fp(4));
    context.recent_anywhere = vec![fp(4)];

    list.append(&[ana(), bia(), newcomer], &context);
    assert_eq!(usable(&list), vec![fp(1), fp(2), fp(4)]);
    assert_eq!(list.selected, Some(fp(1)));
}

#[test]
fn existing_rows_keep_their_place_when_the_ordering_inputs_change() {
    let mut list = list_of(&[ana(), bia(), carla()]);
    let mut context = context();
    context.last_used_here = Some(fp(3));
    context.recent_anywhere = vec![fp(3), fp(2)];
    list.append(&[carla(), bia(), ana()], &context);
    assert_eq!(usable(&list), vec![fp(1), fp(2), fp(3)]);
}

#[test]
fn several_newcomers_are_appended_in_listing_order_rules() {
    let mut list = list_of(&[ana()]);
    list.append(&[ana(), carla(), bia()], &context());
    assert_eq!(usable(&list), vec![fp(1), fp(2), fp(3)]);
}

#[test]
fn appending_the_same_listing_again_changes_nothing() {
    let mut list = list_of(&[ana(), bia()]);
    let before = list.clone();
    list.append(&[ana(), bia()], &context());
    assert_eq!(list, before);
}

#[test]
fn a_new_disabled_certificate_goes_to_the_end_of_the_disabled_group() {
    let expired = |n, name: &str| with_info(candidate(n, name), |i| i.not_after = noon(2026, 1, 1));
    let mut list = list_of(&[ana(), expired(5, "Zeca")]);
    assert_eq!(disabled(&list), vec![fp(5)]);
    list.append(&[ana(), expired(5, "Zeca"), expired(6, "Abel")], &context());
    assert_eq!(disabled(&list), vec![fp(5), fp(6)]);
    assert_eq!(usable(&list), vec![fp(1)]);
}

#[test]
fn a_new_hidden_certificate_is_recorded_as_hidden() {
    let mut list = list_of(&[ana()]);
    let mut no_key = candidate(9, "Sem Chave");
    no_key.has_private_key = false;
    list.append(&[ana(), no_key], &context());
    assert_eq!(usable(&list), vec![fp(1)]);
    assert_eq!(
        hidden_reason(&list, fp(9)),
        Some(HiddenReason::NoPrivateKey)
    );
}

#[test]
fn a_removed_selected_row_is_disabled_in_place_and_stays_selected() {
    let mut list = list_of(&[ana(), bia(), carla()]);
    let mut gone = ana();
    gone.removed = true;
    list.append(&[gone, bia(), carla()], &context());

    assert_eq!(
        status(&list, fp(1)),
        RowStatus::Disabled(DisabledReason::Removed)
    );
    assert_eq!(list.selected, Some(fp(1)));
    assert_eq!(status(&list, fp(2)), RowStatus::Usable);
    assert_eq!(status(&list, fp(3)), RowStatus::Usable);
    let others: Vec<_> = usable(&list).into_iter().filter(|f| *f != fp(1)).collect();
    assert_eq!(others, vec![fp(2), fp(3)]);
}

#[test]
fn a_returning_token_makes_its_row_usable_and_selected_again() {
    let mut list = list_of(&[ana(), bia()]);
    let mut gone = ana();
    gone.removed = true;
    list.append(&[gone, bia()], &context());
    list.append(&[ana(), bia()], &context());

    assert_eq!(status(&list, fp(1)), RowStatus::Usable);
    assert_eq!(list.selected, Some(fp(1)));
    assert_eq!(usable(&list), vec![fp(1), fp(2)]);
}

#[test]
fn a_removed_row_that_was_not_selected_is_disabled_and_the_selection_holds() {
    let mut list = list_of(&[ana(), bia()]);
    let mut gone = bia();
    gone.removed = true;
    list.append(&[ana(), gone], &context());
    assert_eq!(
        status(&list, fp(2)),
        RowStatus::Disabled(DisabledReason::Removed)
    );
    assert_eq!(list.selected, Some(fp(1)));
}

#[test]
fn appending_to_an_empty_list_selects_like_a_fresh_build() {
    let mut list = list_of(&[]);
    assert_eq!(list.selected, None);
    list.append(&[bia(), ana()], &context());
    assert_eq!(usable(&list), vec![fp(1), fp(2)]);
    // A list with no selection has no choice of the person to keep, so the
    // first usable row is chosen, as a fresh build would.
    assert_eq!(list.selected, Some(fp(1)));
}

#[test]
fn a_row_missing_from_a_later_listing_stays_as_it_was() {
    let mut list = list_of(&[ana(), bia()]);
    list.append(&[bia()], &context());
    assert_eq!(usable(&list), vec![fp(1), fp(2)]);
    assert_eq!(status(&list, fp(1)), RowStatus::Usable);
    assert_eq!(list.selected, Some(fp(1)));
}
