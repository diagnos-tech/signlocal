//! PIN outcomes, the empty list, choose mode, the PIN blocks and a token
//! removed while selected.

use websign_protocol::ErrorCode;

use super::rig::{KEY, Rig, SIGN, code, pin_app};
use super::*;
use crate::certs::PinMode;
use crate::confirm::port::{Failure, Finish, Mode};
use crate::confirm::view::{CodeCard, FooterHint, PinBlock, PinError, PinSystem, PrimaryButton};
use crate::fixtures::{candidate, fingerprint};

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
    assert_eq!(rig.view().pin, PinBlock::OsPrompt { now: false, system: PinSystem::Windows });
    assert_eq!(rig.view().footer.hint, FooterHint::OsPinPrompt);
    assert!(rig.view().footer.cancel_enabled);
    rig.click();
    assert!(!rig.view().footer.cancel_enabled);
    assert!(rig.input(UserInput::Escape).is_empty());
    assert_eq!(rig.view().pin, PinBlock::OsPrompt { now: true, system: PinSystem::Windows });
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
