//! "Getting started" (`docs/ux.md` §8.2): a first run lists exactly what
//! is missing with one click each, a ready computer says so and offers
//! the test page, and both read well to screen readers.

use egui::accesskit::Role;
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use websign_protocol::messages::DiagnosticsTab;

use super::fixture;
use super::support::{Setup, open};

fn first_run() -> super::support::Window {
    open(Setup {
        facts: Some(fixture::first_run()),
        tab: Some(DiagnosticsTab::Browsers),
        height: 1000.0,
        ..Setup::default()
    })
}

#[test]
fn a_first_run_says_what_is_missing_and_how_to_fix_it() {
    let mut window = first_run();
    let h = &window.harness;
    h.get_by_role_and_label(Role::Heading, "Getting started");
    h.get_by_label("2 of 6 done");
    h.get_by_label("App installed");
    h.get_by_label("Card service running");
    h.get_by_role_and_label(
        Role::ListItem,
        "Add the browser extension, Not connected yet in Google Chrome, Firefox.",
    );
    h.get_by_role_and_label(
        Role::ListItem,
        "Install the token driver, \
         SafeNet eToken 5110 needs SafeNet Authentication Client to show its certificates.",
    );
    h.get_by_label_contains("Connect your certificate");
    h.get_by_role_and_label(Role::Button, "Test your setup");

    h.get_by_role_and_label(Role::Link, "Already installed? Finish setting up")
        .click();
    window.harness.run();
    let os = if cfg!(windows) {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Linux"
    };
    window
        .harness
        .get_by_role_and_label(Role::Button, &format!("Download for {os}"))
        .click();
    window.harness.run();
    let urls = window.os.borrow().urls.clone();
    assert!(urls[0].ends_with("/activate/"), "{urls:?}");
    assert_eq!(urls[1], "https://example.com/sac");
}

#[test]
fn a_stopped_card_service_offers_its_command() {
    let mut facts = fixture::first_run();
    facts.devices.pcscd_running = Some(false);
    let mut window = open(Setup {
        facts: Some(facts),
        tab: Some(DiagnosticsTab::Browsers),
        height: 1000.0,
        ..Setup::default()
    });
    window
        .harness
        .get_by_label_contains("Start the card service");
    window
        .harness
        .get_by_role_and_label(Role::Button, "Copy command")
        .click();
    window.harness.step();
    let copied = window.harness.output().platform_output.commands.clone();
    assert!(
        format!("{copied:?}").contains("sudo systemctl enable --now pcscd.socket"),
        "{copied:?}"
    );
}

#[test]
fn a_computer_that_can_sign_is_ready_and_offers_the_test_page() {
    let mut window = open(Setup::default());
    window
        .harness
        .get_by_role_and_label(Role::Heading, "You're ready to sign");
    assert!(
        window
            .harness
            .query_by_label_contains("Getting started")
            .is_none()
    );
    window
        .harness
        .get_by_role_and_label(Role::Button, "Test your setup")
        .click();
    window.harness.run();
    let urls = window.os.borrow().urls.clone();
    assert!(urls[0].ends_with("/test/"), "{urls:?}");
    window
        .harness
        .get_by_role_and_label(Role::Button, "Hide")
        .click();
    window.harness.run();
    assert!(window.window().settings.onboarding_dismissed);
    assert!(
        window
            .harness
            .query_by_label("You're ready to sign")
            .is_none()
    );
}

#[test]
fn every_fix_is_reached_with_tab_in_reading_order() {
    let mut window = first_run();
    let mut order = Vec::new();
    for _ in 0..16 {
        window.harness.key_press(egui::Key::Tab);
        window.harness.run();
        let label = window
            .harness
            .query_by(|node| node.is_focused())
            .and_then(|node| node.accesskit_node().label())
            .unwrap_or_default();
        order.push(label);
    }
    let at = |wanted: &str| {
        order
            .iter()
            .position(|label| label.starts_with(wanted))
            .unwrap_or_else(|| panic!("{wanted} not reached: {order:#?}"))
    };
    let fixes = [
        at("Hide"),
        at("Install extension"),
        at("Download for"),
        at("Test your setup"),
        at("Already installed? Finish setting up"),
    ];
    assert!(fixes.is_sorted(), "{order:#?}");
}
