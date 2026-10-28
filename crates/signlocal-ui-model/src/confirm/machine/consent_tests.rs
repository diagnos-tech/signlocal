//! Per-certificate consent (D11): a remembered caller skips Continue only
//! for the certificates its consent covers. Also the two timeouts.

use super::rig::{KEY, Rig, SIGN, request};
use super::*;
use crate::confirm::port::Finish;
use crate::confirm::view::{CodeCard, Expiry, PrimaryButton, RememberBox};
use crate::fixtures::{candidate, context, fingerprint};

/// A remembered caller whose consent covers only `fingerprint(1)`.
fn remembered_for_one(rig: &mut Rig, selected_first: u8) {
    let mut open = request(SIGN, true);
    open.consented = vec![fingerprint(1)];
    rig.apply(UiCommand::Open(open));
    rig.input(UserInput::Focus(true));
    let mut context = context();
    context.requested = Some(fingerprint(selected_first));
    rig.apply(UiCommand::Certificates {
        key: KEY,
        candidates: vec![candidate(1, "Ana"), candidate(2, "Bia")],
        possible: Vec::new(),
        context,
    });
}

#[test]
fn a_certificate_outside_the_consent_needs_continue() {
    let mut rig = Rig::new();
    remembered_for_one(&mut rig, 2);
    let view = rig.view();
    assert_eq!(view.selected, Some(fingerprint(2)));
    assert_eq!(view.code, CodeCard::ContinueHint);
    assert_eq!(view.footer.primary, PrimaryButton::Continue);
    assert!(view.header.remembered, "the chip still says Allowed site");
    assert_eq!(view.remember, RememberBox::Enabled { checked: false });
    assert_eq!(rig.click(), [Intent::Continue(fingerprint(2))]);
}

#[test]
fn moving_to_the_consented_certificate_waits_for_its_digest() {
    let mut rig = Rig::new();
    remembered_for_one(&mut rig, 2);
    rig.wait(700);
    assert_eq!(
        rig.input(UserInput::Select(fingerprint(1))),
        [Intent::Selected(fingerprint(1))]
    );
    let view = rig.view();
    assert!(matches!(view.code, CodeCard::Preparing { .. }));
    assert_eq!(view.remember, RememberBox::Hidden);
    rig.wait(700);
    assert_eq!(
        rig.input(UserInput::Select(fingerprint(2))),
        [Intent::Selected(fingerprint(2))]
    );
    assert_eq!(rig.view().code, CodeCard::ContinueHint);
}

#[test]
fn remember_adds_a_new_certificate_for_a_remembered_caller() {
    let mut rig = Rig::new();
    remembered_for_one(&mut rig, 2);
    rig.wait(700);
    rig.input(UserInput::Remember(true));
    assert_eq!(rig.click(), [Intent::Continue(fingerprint(2))]);
    rig.digest_ready(2);
    assert_eq!(
        rig.click(),
        [Intent::Sign {
            fingerprint: fingerprint(2),
            via: 0,
            remember: true
        }]
    );
}

#[test]
fn the_notice_tells_which_wait_ran_out() {
    for (finish, expiry) in [
        (Finish::Timeout, Expiry::Decision),
        (Finish::DigestTimeout, Expiry::Digest),
    ] {
        let mut rig = Rig::new();
        rig.open_default(SIGN, false);
        assert_eq!(rig.view().expiry, None);
        rig.finish(finish);
        assert_eq!(*rig.model.state(), ConfirmState::Timeout);
        assert_eq!(rig.view().expiry, Some(expiry));
    }
}
