//! Keys (`docs/ux.md` §4.4, §4.9): Esc always cancels, Enter never releases
//! nor chooses a certificate, Space on the focused button does, arrows move
//! the selection and Enter on a row only moves focus.

use egui::Key;
use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use websign_protocol::ErrorCode;
use websign_protocol::types::BrowserName;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::Mode;

use super::fixtures::*;
use super::support::Rig;

fn open(rig: &mut Rig, mode: Mode, remembered: bool) {
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(mode, caller, remembered));
    rig.list(vec![ana_a3(), ana_a1()], Vec::new());
}

fn focused_label(rig: &Rig) -> String {
    rig.harness
        .query_by(|node| node.is_focused())
        .and_then(|node| node.accesskit_node().label())
        .unwrap_or_default()
}

#[test]
fn escape_cancels_even_before_arming() {
    let mut rig = Rig::new(false);
    open(&mut rig, SIGN, false);
    rig.harness.key_press(Key::Escape);
    rig.settle();
    let sent = rig.sent();
    assert!(
        matches!(
            sent.as_slice(),
            [UiEvent::Cancel {
                code: ErrorCode::UserCancelled,
                ..
            }]
        ),
        "{sent:?}"
    );
}

#[test]
fn escape_on_an_empty_list_says_no_certificates() {
    let mut rig = Rig::new(false);
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(SIGN, caller, true));
    rig.list(Vec::new(), Vec::new());
    assert_eq!(focused_label(&rig), "Cancel", "focus on Cancel (§4.8)");
    rig.harness.key_press(Key::Escape);
    rig.settle();
    let sent = rig.sent();
    assert!(
        matches!(
            sent.as_slice(),
            [UiEvent::Cancel {
                code: ErrorCode::NoCertificates,
                ..
            }]
        ),
        "{sent:?}"
    );
}

#[test]
fn enter_never_continues_but_space_on_the_button_does() {
    let mut rig = Rig::new(false);
    open(&mut rig, SIGN, false);
    rig.wait(700);
    assert!(
        focused_label(&rig).starts_with("Ana Beatriz Souza"),
        "initial focus on the row"
    );
    rig.harness.key_press(Key::Enter);
    rig.settle();
    assert_eq!(
        focused_label(&rig),
        "Continue",
        "Enter on a row moves focus on"
    );
    rig.harness.key_press(Key::Enter);
    rig.settle();
    assert!(
        rig.sent().is_empty(),
        "Enter never releases the certificate"
    );
    rig.harness.key_press(Key::Space);
    rig.settle();
    let sent = rig.sent();
    assert!(
        matches!(sent.as_slice(), [UiEvent::Continue { .. }]),
        "{sent:?}"
    );
}

#[test]
fn enter_never_chooses() {
    let mut rig = Rig::new(false);
    open(&mut rig, Mode::Choose, false);
    rig.wait(700);
    rig.harness
        .get_by_role_and_label(Role::Button, "Use this certificate")
        .focus();
    rig.settle();
    rig.harness.key_press(Key::Enter);
    rig.settle();
    assert!(rig.sent().is_empty());
}

#[test]
fn enter_on_the_focused_sign_button_signs() {
    let mut rig = Rig::new(false);
    open(&mut rig, SIGN, true);
    rig.digest(1);
    rig.wait(700);
    rig.harness
        .get_by_role_and_label(Role::Button, "Sign")
        .focus();
    rig.settle();
    rig.harness.key_press(Key::Enter);
    rig.settle();
    let sent = rig.sent();
    assert!(
        matches!(sent.as_slice(), [UiEvent::Sign { .. }]),
        "{sent:?}"
    );
}

#[test]
fn arrows_move_the_selection_over_rows_that_can_sign() {
    let mut rig = Rig::new(false);
    open(&mut rig, SIGN, false);
    rig.wait(700);
    rig.harness.key_press(Key::ArrowDown);
    rig.settle();
    let sent = rig.sent();
    assert!(
        matches!(sent.as_slice(), [UiEvent::Selected { fingerprint: chosen, .. }] if *chosen == fingerprint(2)),
        "{sent:?}"
    );
    assert!(
        focused_label(&rig).contains("ICP-Brasil A1"),
        "focus follows"
    );
    rig.wait(700);
    rig.harness.key_press(Key::ArrowDown);
    rig.settle();
    assert!(rig.sent().is_empty(), "no row below the last one");
    rig.wait(700);
    rig.harness.key_press(Key::Home);
    rig.settle();
    assert!(matches!(rig.sent().as_slice(), [UiEvent::Selected { .. }]));
}

#[test]
fn arrows_are_not_held_back_by_the_rearm_they_cause() {
    let mut rig = Rig::new(false);
    open(&mut rig, SIGN, false);
    rig.harness.key_press(Key::ArrowDown);
    rig.settle();
    assert!(rig.sent().is_empty(), "a window just shown drops arrows");
    rig.wait(700);
    for (key, seed) in [(Key::ArrowDown, 2), (Key::ArrowUp, 1), (Key::End, 2)] {
        rig.wait(50);
        rig.harness.key_press(key);
        rig.settle();
        let sent = rig.sent();
        assert!(
            matches!(sent.as_slice(), [UiEvent::Selected { fingerprint: chosen, .. }] if *chosen == fingerprint(seed)),
            "{key:?}: {sent:?}"
        );
    }
    rig.harness.key_press(Key::Enter);
    rig.settle();
    rig.harness.key_press(Key::Space);
    rig.settle();
    assert!(rig.sent().is_empty(), "Continue still waits for its re-arm");
}
