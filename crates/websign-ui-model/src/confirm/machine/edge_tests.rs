//! Result screens, the timeout countdown and the repaint deadlines.

use std::time::Duration;

use super::rig::{Rig, SIGN};
use super::*;
use crate::confirm::port::Finish;
use crate::confirm::view::FooterHint;

fn signing_rig() -> Rig {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    rig.digest_ready(1);
    rig
}

#[test]
fn site_cancelled_holds_one_and_a_half_seconds() {
    let mut rig = signing_rig();
    rig.finish(Finish::SiteCancelled);
    assert_eq!(*rig.model.state(), ConfirmState::SiteCancelled);
    rig.wait(1499);
    rig.model.tick(rig.now());
    assert_eq!(*rig.model.state(), ConfirmState::SiteCancelled);
    rig.wait(1);
    rig.model.tick(rig.now());
    assert_eq!(*rig.model.state(), ConfirmState::Idle);
}

#[test]
fn timeout_and_abort() {
    let mut rig = signing_rig();
    rig.finish(Finish::Timeout);
    assert_eq!(*rig.model.state(), ConfirmState::Timeout);
    assert!(rig.input(UserInput::Escape).is_empty());
    assert_eq!(*rig.model.state(), ConfirmState::Idle);
    let mut rig = signing_rig();
    rig.finish(Finish::Aborted);
    assert_eq!(*rig.model.state(), ConfirmState::Idle);
}

#[test]
fn countdown_shows_in_the_last_thirty_seconds() {
    let mut rig = signing_rig();
    assert_eq!(rig.view().footer.hint, FooterHint::OsPinPrompt);
    rig.wait(269_000);
    assert_eq!(rig.view().footer.hint, FooterHint::OsPinPrompt);
    let start = rig.now();
    assert_eq!(
        rig.model.next_deadline(start),
        Some(start + Duration::from_secs(1))
    );
    rig.wait(1000);
    assert_eq!(
        rig.view().footer.hint,
        FooterHint::ExpiresIn { seconds: 30 }
    );
    rig.wait(1500);
    assert_eq!(
        rig.view().footer.hint,
        FooterHint::ExpiresIn { seconds: 29 }
    );
    let now = rig.now();
    assert_eq!(
        rig.model.next_deadline(now),
        Some(now + Duration::from_millis(500))
    );
}

#[test]
fn arming_and_skeleton_are_deadlines() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    let t0 = rig.now();
    assert_eq!(
        rig.model.next_deadline(t0),
        Some(t0 + Duration::from_millis(150))
    );
    rig.wait(150);
    assert_eq!(
        rig.model.next_deadline(rig.now()),
        Some(t0 + Duration::from_millis(600))
    );
    rig.wait(500);
    assert!(
        rig.model
            .next_deadline(rig.now())
            .is_some_and(|at| at > rig.now())
    );
    assert_eq!(ConfirmModel::new().next_deadline(t0), None);
}
