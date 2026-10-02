//! The main flows: new and remembered callers, arming, focus, selection.

use websign_protocol::ErrorCode;

use super::rig::{Rig, SIGN};
use super::*;
use crate::confirm::port::Finish;
use crate::confirm::view::{CodeCard, PrimaryButton, RememberBox};
use crate::fixtures::fingerprint;

#[test]
fn new_caller_continues_then_signs() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, false);
    assert_eq!(*rig.model.state(), ConfirmState::Choosing);
    let view = rig.view();
    assert_eq!(view.code, CodeCard::ContinueHint);
    assert_eq!(view.footer.primary, PrimaryButton::Continue);
    assert!(!view.footer.primary_enabled);
    assert_eq!(view.selected, Some(fingerprint(1)));

    rig.wait(600);
    assert!(rig.view().footer.primary_enabled);
    assert_eq!(rig.click(), [Intent::Continue(fingerprint(1))]);
    assert!(matches!(
        rig.view().code,
        CodeCard::Preparing { skeleton: false }
    ));

    rig.wait(200);
    assert!(matches!(
        rig.view().code,
        CodeCard::Preparing { skeleton: true }
    ));

    rig.digest_ready(1);
    assert_eq!(*rig.model.state(), ConfirmState::Ready);
    assert!(!rig.view().footer.primary_enabled, "digest change re-arms");
    assert_eq!(rig.view().footer.primary, PrimaryButton::Sign);
    assert_eq!(
        rig.click(),
        [Intent::Sign {
            fingerprint: fingerprint(1),
            via: 0,
            remember: false
        }]
    );
    assert_eq!(*rig.model.state(), ConfirmState::Signing);
    assert_eq!(rig.view().footer.primary, PrimaryButton::Signing);

    rig.finish(Finish::Signed);
    assert_eq!(*rig.model.state(), ConfirmState::Success);
    rig.wait(899);
    rig.model.tick(rig.now());
    assert_eq!(*rig.model.state(), ConfirmState::Success);
    rig.wait(1);
    rig.model.tick(rig.now());
    assert_eq!(*rig.model.state(), ConfirmState::Idle);
}

#[test]
fn remembered_caller_waits_for_the_digest_not_for_continue() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    let view = rig.view();
    assert!(matches!(view.code, CodeCard::Preparing { .. }));
    assert_eq!(view.footer.primary, PrimaryButton::Sign);
    assert_eq!(view.remember, RememberBox::Hidden);
    assert!(rig.click().is_empty(), "no digest yet");
    rig.digest_ready(1);
    assert!(matches!(rig.view().code, CodeCard::Ready { .. }));
}

#[test]
fn input_is_discarded_while_unarmed_except_escape() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, false);
    rig.wait(100);
    assert!(rig.input(UserInput::Enter).is_empty());
    assert!(rig.input(UserInput::Select(fingerprint(2))).is_empty());
    rig.input(UserInput::Remember(true));
    rig.input(UserInput::PrimaryPress);
    rig.wait(600);
    assert!(
        rig.input(UserInput::PrimaryRelease).is_empty(),
        "press was unarmed"
    );
    assert_eq!(rig.view().selected, Some(fingerprint(1)));
    assert_eq!(rig.view().remember, RememberBox::Enabled { checked: false });
    let mut rig = Rig::new();
    rig.open_default(SIGN, false);
    assert_eq!(
        rig.input(UserInput::Escape),
        [Intent::Cancel(ErrorCode::UserCancelled)]
    );
}

#[test]
fn focus_loss_disarms_and_gain_rearms() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, false);
    rig.wait(700);
    assert!(rig.view().footer.primary_enabled);
    rig.input(UserInput::Focus(false));
    rig.wait(5000);
    assert!(!rig.view().footer.primary_enabled);
    rig.input(UserInput::Focus(true));
    rig.wait(599);
    assert!(!rig.view().footer.primary_enabled);
    rig.wait(1);
    assert!(rig.view().footer.primary_enabled);
}

#[test]
fn selecting_another_certificate_returns_to_choosing() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    rig.digest_ready(1);
    rig.wait(700);
    assert_eq!(
        rig.input(UserInput::Select(fingerprint(2))),
        [Intent::Selected(fingerprint(2))]
    );
    assert_eq!(*rig.model.state(), ConfirmState::Choosing);
    assert!(matches!(rig.view().code, CodeCard::Preparing { .. }));
    assert!(!rig.view().footer.primary_enabled);
    rig.digest_ready(1);
    assert_eq!(*rig.model.state(), ConfirmState::Choosing, "stale digest");
    rig.digest_ready(2);
    assert_eq!(*rig.model.state(), ConfirmState::Ready);
}

#[test]
fn new_caller_selection_shows_the_hint_again() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, false);
    rig.wait(700);
    rig.input(UserInput::Select(fingerprint(2)));
    assert_eq!(rig.view().code, CodeCard::ContinueHint);
    assert_eq!(rig.view().selected, Some(fingerprint(2)));
}
