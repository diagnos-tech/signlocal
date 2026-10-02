//! Arming as the renderer sees it (`is_armed`, `accepts_selection`), moving
//! the selection during a re-arm, and the loading hints.

use super::rig::{KEY, Rig, SIGN};
use super::*;
use crate::fixtures::fingerprint;

#[test]
fn is_armed_counts_600_ms_of_focus() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    rig.wait(599);
    assert!(!rig.model.is_armed(rig.now()));
    rig.wait(1);
    assert!(rig.model.is_armed(rig.now()));
    rig.input(UserInput::Focus(false));
    assert!(!rig.model.is_armed(rig.now()));
}

#[test]
fn a_selection_is_gated_only_until_the_window_first_arms() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, false);
    rig.wait(100);
    assert!(!rig.model.accepts_selection(rig.now()));
    assert!(rig.input(UserInput::Select(fingerprint(2))).is_empty());
    rig.wait(600);
    assert!(rig.model.accepts_selection(rig.now()));
    assert_eq!(
        rig.input(UserInput::Select(fingerprint(2))),
        vec![Intent::Selected(fingerprint(2))]
    );
    // The selection re-armed Sign, but the next arrow is not held back.
    rig.wait(50);
    assert!(!rig.model.is_armed(rig.now()));
    assert_eq!(
        rig.input(UserInput::Select(fingerprint(1))),
        vec![Intent::Selected(fingerprint(1))]
    );
    // Only the primary button waits for arming again.
    assert!(rig.input(UserInput::PrimaryPress).is_empty());
}

#[test]
fn losing_focus_or_a_new_request_gates_the_selection_again() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, false);
    rig.wait(700);
    rig.input(UserInput::Focus(false));
    rig.input(UserInput::Focus(true));
    assert!(rig.input(UserInput::Select(fingerprint(2))).is_empty());
    rig.wait(700);
    assert!(rig.model.accepts_selection(rig.now()));
    rig.open_default(SIGN, false);
    assert!(!rig.model.accepts_selection(rig.now()));
}

#[test]
fn the_digest_rearming_an_armed_window_keeps_the_selection_open() {
    let mut rig = Rig::new();
    rig.open_default(SIGN, true);
    rig.wait(700);
    rig.digest_ready(1);
    assert!(!rig.model.is_armed(rig.now()));
    assert!(rig.model.accepts_selection(rig.now()));
}

#[test]
fn loading_shows_the_skeleton_then_the_slow_device() {
    let mut rig = Rig::new();
    rig.apply(UiCommand::Open(super::rig::request(SIGN, true)));
    let loading = |rig: &Rig| rig.view().loading.expect("loading");
    assert!(!loading(&rig).skeleton);
    assert_eq!(
        rig.model.next_deadline(rig.now()),
        Some(rig.now() + ms(150))
    );
    rig.wait(150);
    assert!(loading(&rig).skeleton);
    rig.apply(UiCommand::SlowListing {
        key: KEY,
        device: Some("SafeNet eToken 5110".into()),
    });
    assert_eq!(loading(&rig).slow_device, None, "not before 2 s");
    rig.wait(1850);
    assert_eq!(
        loading(&rig).slow_device.as_deref(),
        Some("SafeNet eToken 5110")
    );
    rig.apply(UiCommand::Certificates {
        key: KEY,
        candidates: Vec::new(),
        possible: Vec::new(),
        context: crate::fixtures::context(),
    });
    assert_eq!(rig.view().loading, None);
}

#[test]
fn scanning_again_starts_a_fresh_listing() {
    let mut rig = Rig::new();
    rig.apply(UiCommand::Open(super::rig::request(SIGN, true)));
    rig.input(UserInput::Focus(true));
    rig.apply(UiCommand::SlowListing {
        key: KEY,
        device: Some("eToken".into()),
    });
    rig.apply(UiCommand::Certificates {
        key: KEY,
        candidates: Vec::new(),
        possible: Vec::new(),
        context: crate::fixtures::context(),
    });
    rig.wait(3000);
    assert_eq!(rig.input(UserInput::Rescan), vec![Intent::Rescan]);
    let loading = rig.view().loading.expect("loading again");
    assert!(!loading.skeleton);
    assert_eq!(loading.slow_device, None);
}

fn ms(value: u64) -> std::time::Duration {
    std::time::Duration::from_millis(value)
}
