//! What screen readers get (`docs/ux.md` §14): the header as one sentence,
//! the list as radio buttons with full names, the PIN as a password field
//! with no value, errors as alerts, and Tab in visual order.

use egui::Key;
use egui::accesskit::{Live, Role, Toggled};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use websign_protocol::types::BrowserName;
use websign_ui_model::confirm::port::Failure;

use super::fixtures::*;
use super::support::Rig;

fn open_site(rig: &mut Rig, remembered: bool) {
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(SIGN, caller, remembered));
}

#[test]
fn the_header_is_read_as_one_sentence() {
    let mut rig = Rig::new(false);
    open_site(&mut rig, false);
    rig.list(vec![ana_a3()], Vec::new());
    rig.harness.get_by_label(
        "Signature request · via Chrome, app.diagnos.health, \
         First time this site asks for anything on this computer",
    );
}

#[test]
fn rows_are_radio_buttons_with_their_whole_story() {
    let mut rig = Rig::new(false);
    open_site(&mut rig, true);
    rig.list(vec![ana_a3(), ana_a1(), old_a3()], Vec::new());
    let row = rig.harness.get_by_role_and_label(
        Role::RadioButton,
        "Ana Beatriz Souza, ICP-Brasil A3, CPF partially hidden, 456 789, \
         AC SOLUTI Multipla v5, Card in reader, Expires in 23 days",
    );
    assert_eq!(row.accesskit_node().toggled(), Some(Toggled::True));
    assert_eq!(rig.harness.get_all_by_role(Role::RadioButton).count(), 2);
    assert!(
        rig.harness.query_by_role(Role::Image).is_none(),
        "no decorative images"
    );
}

#[test]
fn the_pin_is_a_password_field_without_value() {
    let mut rig = Rig::new(false);
    open_site(&mut rig, true);
    rig.list(vec![ana_token(app_pin(false, false, false))], Vec::new());
    rig.digest(5);
    rig.wait(700);
    rig.harness
        .get_by_role_and_label(Role::PasswordInput, "Token PIN")
        .type_text("1234");
    rig.settle();
    let field = rig.harness.get_by_role(Role::PasswordInput);
    assert_eq!(
        field.accesskit_node().value(),
        None,
        "not the PIN, not its length"
    );
    assert_eq!(rig.window().typed_pin_len(), 4);
}

#[test]
fn errors_are_announced_at_once() {
    let mut rig = Rig::new(false);
    open_site(&mut rig, true);
    rig.list(vec![ana_a3()], Vec::new());
    rig.digest(1);
    rig.fail(Failure::TokenRemoved);
    let alert = rig.harness.get_by_role(Role::Alert);
    assert_eq!(alert.accesskit_node().live(), Live::Assertive);
    assert!(
        alert
            .accesskit_node()
            .label()
            .is_some_and(|label| label.starts_with("The token was removed")),
    );
}

#[test]
fn tab_follows_the_visual_order_and_never_starts_on_sign() {
    let mut rig = Rig::new(false);
    open_site(&mut rig, false);
    rig.list(vec![ana_a3(), ana_a1(), old_a3()], Vec::new());
    rig.wait(700);
    let mut order = Vec::new();
    for _ in 0..8 {
        let label = rig
            .harness
            .query_by(|node| node.is_focused())
            .and_then(|node| node.accesskit_node().label())
            .unwrap_or_default();
        order.push(label);
        rig.harness.key_press(Key::Tab);
        rig.settle();
    }
    assert!(
        order[0].starts_with("Ana Beatriz Souza, ICP-Brasil A3"),
        "{order:?}"
    );
    let at = |wanted: &str| order.iter().position(|label| label.starts_with(wanted));
    let (cancel, primary) = (at("Cancel"), at("Continue"));
    let remember = at("Remember this site");
    assert!(remember.is_some() && remember < cancel, "{order:?}");
    if cfg!(windows) {
        assert!(primary < cancel, "Windows puts the primary button first");
    } else {
        assert!(cancel < primary, "{order:?}");
    }
}

#[test]
fn the_pin_hint_names_the_key_store_never_the_running_os() {
    let mut rig = Rig::new(false);
    open_site(&mut rig, true);
    rig.list(vec![ana_a3()], Vec::new());
    rig.harness
        .get_by_label("Windows will ask for your PIN in its own window.");
    let mut rig = Rig::new(false);
    open_site(&mut rig, true);
    let driver = ana_token(websign_ui_model::certs::PinMode::System);
    rig.list(vec![driver], Vec::new());
    assert!(
        rig.harness
            .query_by_label_contains("will ask for your PIN")
            .is_none(),
        "a token driver has no system dialog: our field asks"
    );
    rig.harness.get_by_role(Role::PasswordInput);
}
