//! ux §4.8 and §6.2, SPEC §2.2: the empty state, tokens coming and going
//! while the window is open.

mod common;

use common::*;
use websign_ui_model::certs::{DisabledReason, RowStatus};
use websign_ui_model::confirm::port::UiCommand;
use websign_ui_model::confirm::view::{CodeCard, FooterHint, PinBlock};
use websign_ui_model::confirm::{ConfirmState, Intent, UserInput};
use websign_ui_model::possible::PossibleCard;

fn safenet() -> PossibleCard {
    PossibleCard {
        name: Some("SafeNet eToken 5110".to_owned()),
        reader: None,
        card: false,
        driver: Some("SafeNet Authentication Client".to_owned()),
        download: Some("https://example.invalid/sac".to_owned()),
        needs_complement: false,
    }
}

fn empty_window(possible: Vec<PossibleCard>) -> Window {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(100).apply(UiCommand::Certificates {
        key: KEY,
        candidates: vec![],
        possible,
        context: context(),
    });
    w.wait(1_000);
    w
}

#[test]
fn a_listing_without_usable_rows_is_the_empty_state() {
    let w = empty_window(vec![]);
    assert_eq!(w.state(), ConfirmState::Empty);
}

#[test]
fn a_listing_of_only_disabled_rows_is_also_empty() {
    let expired = with_info(candidate(1, "Ana"), |i| i.not_after = noon(2026, 1, 1));
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(100).certificates(vec![expired], context());
    assert_eq!(w.state(), ConfirmState::Empty);
    let list = w
        .view()
        .list
        .expect("the list is shown with its disabled group");
    assert_eq!(
        list.disabled[0].status,
        RowStatus::Disabled(DisabledReason::Expired)
    );
}

#[test]
fn the_empty_view_has_no_code_no_selection_and_a_disabled_primary_button() {
    let view = empty_window(vec![]).view();
    assert_eq!(view.code, CodeCard::Hidden);
    assert_eq!(view.selected, None);
    assert_eq!(view.pin, PinBlock::Hidden);
    assert!(!view.footer.primary_enabled);
    assert!(view.footer.cancel_enabled);
    assert_eq!(view.footer.hint, FooterHint::OpenDiagnostics);
}

#[test]
fn possible_certificates_are_passed_to_the_view_untouched() {
    let view = empty_window(vec![safenet()]).view();
    assert_eq!(view.possible, vec![safenet()]);
}

#[test]
fn enter_and_clicks_do_nothing_in_the_empty_state() {
    let mut w = empty_window(vec![]);
    assert_eq!(w.input(UserInput::Enter), vec![]);
    assert_eq!(w.click(), vec![]);
    assert_eq!(w.state(), ConfirmState::Empty);
}

#[test]
fn rescan_and_diagnostics_are_available_in_the_empty_state() {
    let mut w = empty_window(vec![safenet()]);
    assert_eq!(w.input(UserInput::Rescan), vec![Intent::Rescan]);
    assert!(matches!(
        w.input(UserInput::OpenDiagnostics).as_slice(),
        [Intent::OpenDiagnostics(_)]
    ));
}

#[test]
fn a_token_inserted_in_the_empty_state_leads_to_choosing() {
    let mut w = empty_window(vec![safenet()]);
    w.wait(500).certificates(pair(), context());
    assert_eq!(w.state(), ConfirmState::Choosing);
    assert_eq!(w.view().selected, Some(fp(1)));
    assert!(w.view().possible.is_empty());
}

#[test]
fn a_token_inserted_while_choosing_is_appended_and_the_selection_holds() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(100).certificates(pair(), context());
    w.wait(1_000);
    w.input(UserInput::Select(fp(2)));
    w.wait(1_000);

    let mut ctx = context();
    ctx.last_used_here = Some(fp(3));
    let mut all = pair();
    all.push(candidate(3, "Aaron"));
    w.certificates(all, ctx);

    let list = w.view().list.unwrap();
    assert_eq!(usable(&list), vec![fp(1), fp(2), fp(3)]);
    assert_eq!(w.view().selected, Some(fp(2)));
    assert_eq!(w.state(), ConfirmState::Choosing);
}

#[test]
fn the_selected_token_being_removed_disables_its_row_and_signing() {
    let mut w = ready_remembered(pair(), context());
    let mut gone = pair();
    gone[0].removed = true;
    w.certificates(gone, context());
    w.wait(1_000);

    let view = w.view();
    assert_eq!(view.selected, Some(fp(1)), "the row stays selected");
    let list = view.list.as_ref().unwrap();
    assert_eq!(
        status(list, fp(1)),
        RowStatus::Disabled(DisabledReason::Removed)
    );
    assert!(!view.footer.primary_enabled);
    assert_eq!(w.input(UserInput::Enter), vec![]);
    assert_eq!(w.click(), vec![]);
}

#[test]
fn a_returning_token_makes_the_row_usable_again() {
    let mut w = ready_remembered(pair(), context());
    let mut gone = pair();
    gone[0].removed = true;
    w.certificates(gone, context());
    w.wait(100).certificates(pair(), context());
    w.wait(1_000);

    let view = w.view();
    assert_eq!(view.selected, Some(fp(1)));
    assert_eq!(
        status(view.list.as_ref().unwrap(), fp(1)),
        RowStatus::Usable
    );
}

#[test]
fn loading_shows_no_code_and_no_selection() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    let view = w.view();
    assert_eq!(view.state, ConfirmState::LoadingCerts);
    assert_eq!(view.selected, None);
    assert_eq!(view.code, CodeCard::Hidden);
    assert!(!view.footer.primary_enabled);
    assert!(view.footer.cancel_enabled);
}

#[test]
fn a_slow_listing_keeps_loading() {
    let mut w = Window::new();
    w.open(request(KEY, sign_mode(), false));
    w.wait(2_000).apply(UiCommand::SlowListing {
        key: KEY,
        device: Some("SafeNet eToken 5110".to_owned()),
    });
    assert_eq!(w.state(), ConfirmState::LoadingCerts);
}
