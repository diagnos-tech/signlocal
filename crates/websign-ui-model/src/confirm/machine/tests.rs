use websign_protocol::ErrorCode;

use super::rig::{KEY, Rig, SIGN, code, pin_app};
use super::*;
use crate::certs::PinMode;
use crate::confirm::port::{Failure, Finish, Mode};
use crate::confirm::view::{CodeCard, FooterHint, PinBlock, PinError, PrimaryButton, RememberBox};
use crate::fixtures::{candidate, fingerprint};

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

#[test]
fn wrong_pin_then_locked() {
    let mut rig = Rig::new();
    let mut c = candidate(1, "Ana");
    c.pin = pin_app(false);
    rig.open(SIGN, true, vec![c, candidate(2, "Bia")]);
    rig.digest_ready(1);
    rig.wait(700);
    assert!(matches!(
        rig.view().pin,
        PinBlock::Field { valid: false, .. }
    ));
    assert!(!rig.view().footer.primary_enabled);
    rig.input(UserInput::PinLength(6));
    assert!(rig.view().footer.primary_enabled);
    assert_eq!(rig.click().len(), 1);

    rig.apply(UiCommand::Failed {
        key: KEY,
        failure: Failure::PinIncorrect {
            count_low: true,
            final_try: false,
        },
    });
    assert_eq!(*rig.model.state(), ConfirmState::PinError);
    let PinBlock::Field { valid, error, .. } = rig.view().pin else {
        panic!("field expected")
    };
    assert!(!valid);
    assert_eq!(error, Some(PinError::IncorrectLow));
    rig.input(UserInput::PinLength(5));
    assert_eq!(rig.input(UserInput::Enter).len(), 1);

    rig.apply(UiCommand::Failed {
        key: KEY,
        failure: Failure::PinLocked {
            tool: None,
            issuer: "AC".into(),
        },
    });
    assert_eq!(*rig.model.state(), ConfirmState::PinLocked);
    let view = rig.view();
    assert_eq!(view.pin, PinBlock::Locked);
    assert!(view.banner.is_some());
    assert!(!view.footer.primary_enabled);
    assert_eq!(
        rig.input(UserInput::Escape),
        [Intent::Cancel(ErrorCode::PinLocked)]
    );
}

#[test]
fn empty_list_blocks_and_cancels_with_no_certificates() {
    let mut rig = Rig::new();
    rig.open(SIGN, false, Vec::new());
    assert_eq!(*rig.model.state(), ConfirmState::Empty);
    let view = rig.view();
    assert_eq!(view.code, CodeCard::Hidden);
    assert_eq!(view.footer.hint, FooterHint::OpenDiagnostics);
    assert_eq!(
        rig.input(UserInput::CancelButton),
        [Intent::Cancel(ErrorCode::NoCertificates)]
    );
}

#[test]
fn a_token_inserted_in_the_empty_state_makes_choosing() {
    let mut rig = Rig::new();
    rig.open(SIGN, false, Vec::new());
    rig.apply(UiCommand::Certificates {
        key: KEY,
        candidates: vec![candidate(1, "Ana")],
        possible: Vec::new(),
        context: crate::fixtures::context(),
    });
    assert_eq!(*rig.model.state(), ConfirmState::Choosing);
    assert_eq!(rig.view().selected, Some(fingerprint(1)));
}

#[test]
fn choose_mode_sends_the_certificate() {
    let mut rig = Rig::new();
    rig.open_default(Mode::Choose, false);
    let view = rig.view();
    assert_eq!(view.code, CodeCard::SelectShares);
    assert_eq!(view.pin, PinBlock::Hidden);
    assert_eq!(view.footer.primary, PrimaryButton::UseCertificate);
    rig.input(UserInput::Remember(true));
    rig.wait(700);
    rig.input(UserInput::Remember(true));
    assert_eq!(
        rig.click(),
        [Intent::Choose {
            fingerprint: fingerprint(1),
            remember: true
        }]
    );
    assert!(rig.click().is_empty(), "no second decision");
    rig.finish(Finish::Chosen);
    assert_eq!(*rig.model.state(), ConfirmState::Success);
}

#[test]
fn os_prompt_disables_cancel_only_while_signing() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    rig.digest_ready(1);
    assert_eq!(rig.view().pin, PinBlock::OsPrompt { now: false });
    assert_eq!(rig.view().footer.hint, FooterHint::OsPinPrompt);
    assert!(rig.view().footer.cancel_enabled);
    rig.click();
    assert!(!rig.view().footer.cancel_enabled);
    assert!(rig.input(UserInput::Escape).is_empty());
    assert_eq!(rig.view().pin, PinBlock::OsPrompt { now: true });
}

#[test]
fn keypad_and_unlocked_blocks() {
    let mut rig = Rig::new();
    let mut c = candidate(1, "Ana");
    c.pin = PinMode::PinPad;
    let mut u = candidate(2, "Bia");
    u.pin = PinMode::Unlocked;
    rig.open(SIGN, true, vec![c, u]);
    assert_eq!(rig.view().pin, PinBlock::PinPad { now: false });
    rig.wait(700);
    rig.input(UserInput::Select(fingerprint(2)));
    assert_eq!(rig.view().pin, PinBlock::Unlocked);
}

#[test]
fn removing_the_selected_token_disables_sign_in_place() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    rig.digest_ready(1);
    let mut gone = candidate(1, "Ana");
    gone.removed = true;
    rig.apply(UiCommand::Certificates {
        key: KEY,
        candidates: vec![gone, candidate(2, "Bia")],
        possible: Vec::new(),
        context: crate::fixtures::context(),
    });
    rig.wait(700);
    assert_eq!(rig.view().selected, Some(fingerprint(1)));
    assert!(!rig.view().footer.primary_enabled);
    assert!(rig.click().is_empty());
}

#[test]
fn ready_code_carries_the_hash() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    rig.digest_ready(1);
    assert_eq!(
        rig.view().code,
        CodeCard::Ready {
            code: code(),
            hash: websign_protocol::types::HashName::Sha256
        }
    );
}
