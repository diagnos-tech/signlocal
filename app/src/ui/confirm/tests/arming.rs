//! The anti-accident rules (`docs/ux.md` §4.7): the primary button only
//! arms after 600 ms of the window being visible and focused, clicks and
//! keys before that are dropped, and a click that only brings the window
//! forward never approves.

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use secrecy::ExposeSecret;
use websign_protocol::types::BrowserName;
use websign_ui_model::confirm::UiEvent;

use super::fixtures::*;
use super::support::Rig;

fn primary_enabled(rig: &Rig, label: &str) -> bool {
    !rig.harness
        .get_by_role_and_label(Role::Button, label)
        .accesskit_node()
        .is_disabled()
}

fn open_ready(rig: &mut Rig) {
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(SIGN, caller, true));
    rig.list(vec![ana_a3(), ana_a1()], Vec::new());
    rig.digest(1);
}

#[test]
fn sign_is_disabled_until_600_ms_after_the_code_appears() {
    let mut rig = Rig::new(false);
    open_ready(&mut rig);
    assert!(!primary_enabled(&rig, "Sign"));
    rig.wait(599);
    assert!(!primary_enabled(&rig, "Sign"), "599 ms is not enough");
    rig.wait(1);
    assert!(primary_enabled(&rig, "Sign"));
}

#[test]
fn a_window_without_focus_never_arms() {
    let mut rig = Rig::new(false);
    rig.harness.input_mut().focused = false;
    rig.settle();
    open_ready(&mut rig);
    rig.wait(5_000);
    assert!(!primary_enabled(&rig, "Sign"), "unfocused for 5 s");
    rig.harness.input_mut().focused = true;
    rig.settle();
    rig.wait(300);
    assert!(!primary_enabled(&rig, "Sign"), "600 ms count from focus");
    rig.wait(300);
    assert!(primary_enabled(&rig, "Sign"));
}

#[test]
fn a_click_before_arming_does_nothing() {
    let mut rig = Rig::new(false);
    open_ready(&mut rig);
    rig.wait(100);
    rig.harness
        .get_by_role_and_label(Role::Button, "Sign")
        .click();
    rig.settle();
    rig.wait(700);
    assert!(
        rig.sent().is_empty(),
        "the early click was not kept for later"
    );
    rig.harness
        .get_by_role_and_label(Role::Button, "Sign")
        .click();
    rig.settle();
    let sent = rig.sent();
    assert!(
        matches!(
            sent.as_slice(),
            [UiEvent::Sign {
                pin: None,
                via: 0,
                ..
            }]
        ),
        "{sent:?}"
    );
}

#[test]
fn continue_waits_for_arming_too() {
    let mut rig = Rig::new(false);
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(SIGN, caller, false));
    rig.list(vec![ana_a3(), ana_a1()], Vec::new());
    rig.harness
        .get_by_role_and_label(Role::Button, "Continue")
        .click();
    rig.settle();
    assert!(rig.sent().is_empty());
    rig.wait(600);
    rig.harness
        .get_by_role_and_label(Role::Button, "Continue")
        .click();
    rig.settle();
    let sent = rig.sent();
    assert!(
        matches!(sent.as_slice(), [UiEvent::Continue { fingerprint: chosen, .. }] if *chosen == fingerprint(1)),
        "{sent:?}"
    );
}

#[test]
fn keys_typed_before_arming_never_reach_the_pin() {
    let mut rig = Rig::new(false);
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(SIGN, caller, true));
    rig.list(vec![ana_token(app_pin(false, false, false))], Vec::new());
    rig.digest(5);
    let field = rig.harness.get_by_role(Role::PasswordInput);
    assert!(field.is_focused(), "the PIN field gets the initial focus");
    field.type_text("9999");
    rig.settle();
    assert_eq!(rig.window().typed_pin_len(), 0, "stray site keystrokes");
    rig.wait(600);
    rig.harness
        .get_by_role(Role::PasswordInput)
        .type_text("1234");
    rig.settle();
    assert_eq!(rig.window().typed_pin_len(), 4);
    rig.harness.key_press(egui::Key::Enter);
    rig.settle();
    let sent = rig.sent();
    let [UiEvent::Sign { pin: Some(pin), .. }] = sent.as_slice() else {
        panic!("Enter in the PIN field signs once armed: {sent:?}");
    };
    assert_eq!(pin.expose_secret(), "1234");
    assert_eq!(rig.window().typed_pin_len(), 0, "the field is wiped");
}
