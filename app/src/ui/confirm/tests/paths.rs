//! What the window asks of the host beyond a decision: "View in system"
//! for the selected certificate (`docs/ux.md` §5.12), and "Try through the
//! token driver" with the driver's own PIN field (§4.6, §5.11).

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use secrecy::ExposeSecret;
use websign_protocol::types::BrowserName;
use websign_ui_model::certs::{KeyPath, KeySource};
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::Failure;

use super::fixtures::*;
use super::support::Rig;

fn open_remembered(rig: &mut Rig) {
    let caller = web("https://app.diagnos.health", BrowserName::Chrome);
    rig.open(request(SIGN, caller, true));
}

#[test]
fn view_in_system_asks_the_host_for_the_selected_certificate() {
    let mut rig = Rig::new(false);
    open_remembered(&mut rig);
    rig.list(vec![ana_a3()], Vec::new());
    rig.wait(700);
    rig.harness.get_by_label("Details").click();
    rig.settle();
    // The details run below the fold: scroll them in, as a person would.
    rig.harness
        .get_by_role_and_label(Role::Button, "View in system")
        .scroll_to_me();
    rig.settle();
    rig.harness
        .get_by_role_and_label(Role::Button, "View in system")
        .click();
    rig.settle();
    let sent = rig.sent();
    assert!(
        matches!(
            sent.as_slice(),
            [UiEvent::ViewCertificate { key, fingerprint: shown }]
                if *key == KEY && *shown == fingerprint(1)
        ),
        "{sent:?}"
    );
}

#[test]
fn the_token_driver_path_asks_for_our_pin_before_signing() {
    let mut rig = Rig::new(false);
    open_remembered(&mut rig);
    let mut card = ana_a3();
    card.alternates = vec![KeyPath {
        source: KeySource::Driver {
            path: "eTPKCS11.dll".to_owned(),
        },
        pin: app_pin(false, false, false),
    }];
    rig.list(vec![card], Vec::new());
    rig.digest(1);
    rig.wait(700);
    assert!(
        rig.harness.query_by_role(Role::PasswordInput).is_none(),
        "the store shows its own dialog"
    );
    rig.fail(Failure::DriverFailure {
        driver: "Windows".to_owned(),
        native: "SCARD_F_INTERNAL_ERROR (0x80100001)".to_owned(),
        alternate: true,
    });
    rig.wait(700);
    rig.harness
        .get_by_role_and_label(Role::Button, "Try through the token driver")
        .click();
    rig.settle();
    assert!(rig.sent().is_empty(), "no PIN typed yet");
    let field = rig.harness.get_by_role(Role::PasswordInput);
    assert!(field.is_focused(), "the driver's PIN field takes the focus");
    rig.wait(700);
    rig.harness
        .get_by_role(Role::PasswordInput)
        .type_text("1234");
    rig.settle();
    rig.harness
        .get_by_role_and_label(Role::Button, "Sign")
        .click();
    rig.settle();
    let sent = rig.sent();
    let [
        UiEvent::Sign {
            via: 1,
            pin: Some(pin),
            ..
        },
    ] = sent.as_slice()
    else {
        panic!("Sign through the driver with the PIN: {sent:?}");
    };
    assert_eq!(pin.expose_secret(), "1234");
}
