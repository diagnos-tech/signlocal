//! What screen readers and keyboards get: a tab list with lights in the
//! tab names, rows read as one sentence, the accordion's state, and the
//! shortcuts of `docs/ux.md` §8.8.

use egui::accesskit::Role;
use egui::{Key, Modifiers};
use egui_kittest::kittest::{NodeT as _, Queryable as _};
use websign_protocol::messages::DiagnosticsTab;

use super::fixture;
use super::support::{Setup, open};

#[test]
fn tabs_form_a_tab_list_with_their_lights() {
    let window = open(Setup::default());
    let list = window.harness.get_by_role(Role::TabList);
    assert_eq!(
        list.accesskit_node().label().as_deref(),
        Some("Diagnostics — SignLocal")
    );
    let browsers = window
        .harness
        .get_by_role_and_label(Role::Tab, "Browsers, Needs attention");
    assert_eq!(browsers.accesskit_node().is_selected(), Some(true));
    window
        .harness
        .get_by_role_and_label(Role::Tab, "Devices, Needs attention");
    window
        .harness
        .get_by_role_and_label(Role::Tab, "Certificates, Needs attention");
    window.harness.get_by_role_and_label(Role::Tab, "Help");
    window.harness.get_by_label("Needs attention");
}

#[test]
fn the_window_opens_on_the_first_red_tab() {
    let mut facts = fixture::facts();
    facts.devices.pcscd_running = Some(false);
    let window = open(Setup {
        facts: Some(facts),
        ..Setup::default()
    });
    let devices = window
        .harness
        .get_by_role_and_label(Role::Tab, "Devices, Can't sign yet");
    assert_eq!(devices.accesskit_node().is_selected(), Some(true));
    // On its row and in "Getting started".
    let shown = window
        .harness
        .get_all_by_label_contains("sudo systemctl enable --now pcscd.socket");
    assert_eq!(shown.count(), 2);
}

#[test]
fn shortcuts_switch_tabs_and_scan_again() {
    let mut window = open(Setup::default());
    window
        .harness
        .key_press_modifiers(Modifiers::COMMAND, Key::Num3);
    window.harness.run();
    assert_eq!(
        window.window().state.tab,
        Some(DiagnosticsTab::Certificates)
    );
    let before = *window.scans.borrow();
    window.harness.key_press(Key::F5);
    window.harness.run();
    assert_eq!(*window.scans.borrow(), before + 1);
    window
        .harness
        .key_press_modifiers(Modifiers::COMMAND, Key::R);
    window.harness.run();
    assert_eq!(*window.scans.borrow(), before + 2);
}

#[test]
fn arrows_move_between_focused_tabs() {
    let mut window = open(Setup::default());
    window
        .harness
        .get_by_role_and_label(Role::Tab, "Browsers, Needs attention")
        .focus();
    window.harness.run();
    window.harness.key_press(Key::ArrowDown);
    window.harness.run();
    assert_eq!(window.window().state.tab, Some(DiagnosticsTab::Devices));
    let devices = window
        .harness
        .get_by_role_and_label(Role::Tab, "Devices, Needs attention");
    assert!(devices.accesskit_node().is_focused());
    window.harness.key_press(Key::ArrowUp);
    window.harness.run();
    window.harness.key_press(Key::ArrowUp);
    window.harness.run();
    assert_eq!(window.window().state.tab, Some(DiagnosticsTab::Help));
}

#[test]
fn rows_read_as_one_sentence() {
    let window = open(Setup {
        tab: Some(DiagnosticsTab::Devices),
        ..Setup::default()
    });
    window.harness.get_by_role_and_label(
        Role::ListItem,
        "SafeNet eToken 5110 USB 0529:0620, No certificates · install SafeNet Authentication Client",
    );
    window.harness.get_by_role_and_label(
        Role::ListItem,
        "Identiv uTrust 2700 R, Unrecognized card · 1 certificate, ATR 3B:D5:18:FF:81:91:FE:1F:C3:..:..:..:..:..:..",
    );
    let os = if cfg!(windows) {
        "Windows"
    } else if cfg!(target_os = "macos") {
        "macOS"
    } else {
        "Linux"
    };
    window
        .harness
        .get_by_role_and_label(Role::Button, &format!("Download for {os}"));
}

#[test]
fn row_actions_reach_the_os() {
    let mut window = open(Setup::default());
    let mut facts = fixture::facts();
    facts.browsers[1].registration = websign_registration::status::RegistrationState::Missing;
    window.window().set_facts(facts);
    window.harness.run();
    window
        .harness
        .get_by_role_and_label(Role::Button, "Repair")
        .click();
    window.harness.run();
    assert_eq!(window.os.borrow().repairs, 1);
    window
        .harness
        .get_by_label_contains("Repaired. Restart Microsoft Edge.");
}

#[test]
fn the_faq_says_which_answer_is_open() {
    let mut window = open(Setup {
        tab: Some(DiagnosticsTab::Help),
        height: 1000.0,
        ..Setup::default()
    });
    let first = window
        .harness
        .get_by_role_and_label(Role::Button, "My certificate doesn't show up");
    assert_eq!(first.accesskit_node().data().is_expanded(), Some(true));
    let shortcuts = window
        .harness
        .get_by_role_and_label(Role::Button, "Keyboard shortcuts");
    assert_eq!(shortcuts.accesskit_node().data().is_expanded(), Some(false));
    shortcuts.click();
    window.harness.run();
    let command = if cfg!(target_os = "macos") {
        "⌘"
    } else {
        "Ctrl"
    };
    window
        .harness
        .get_by_label_contains(&format!("F5, {command}+R — Scan again"));
    window.harness.get_by_role(Role::Document);
}
