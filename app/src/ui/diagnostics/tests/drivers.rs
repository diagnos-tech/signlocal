//! "Add driver…" (`docs/ux.md` §8.4): the OS file picker chooses the
//! module, which is remembered and loaded by a new scan; without a picker
//! the window asks for the path itself; cancelling changes nothing.

use std::path::PathBuf;

use egui::accesskit::Role;
use egui_kittest::kittest::Queryable as _;
use websign_protocol::messages::DiagnosticsTab;

use super::support::{Setup, Window, open};
use crate::platform::file_picker::Picked;

/// The window after "Add driver…", and how many scans had started before.
fn press_add_driver(answer: Picked) -> (Window, usize) {
    let mut window = open(Setup {
        tab: Some(DiagnosticsTab::Devices),
        height: 1200.0,
        ..Setup::default()
    });
    window.window().settings.onboarding_dismissed = true;
    window.os.borrow_mut().picker_answer = answer;
    window.harness.run();
    let scans = *window.scans.borrow();
    window
        .harness
        .get_by_role_and_label(Role::Button, "Add driver…")
        .click();
    window.harness.run();
    (window, scans)
}

#[test]
fn the_chosen_module_is_remembered_and_scanned() {
    let module = PathBuf::from("/opt/vendor/libvendor-pkcs11.so");
    let (mut window, scans_before) = press_add_driver(Picked::Chosen(module.clone()));
    {
        let log = window.os.borrow();
        assert_eq!(log.pickers.len(), 1);
        assert_eq!(log.pickers[0].title, "Choose the token driver");
        assert_eq!(log.pickers[0].type_name, "Token driver");
    }
    assert_eq!(window.window().settings.user_modules, vec![module]);
    assert!(*window.scans.borrow() > scans_before, "a scan loads it");
    assert!(
        window
            .harness
            .query_by_role_and_label(Role::TextInput, "Path to the driver file")
            .is_none()
    );
}

#[test]
fn without_a_picker_the_path_is_typed() {
    let (mut window, _) = press_add_driver(Picked::Unavailable);
    assert!(window.window().settings.user_modules.is_empty());
    window
        .harness
        .get_by_role_and_label(Role::TextInput, "Path to the driver file");
}

#[test]
fn cancelling_changes_nothing() {
    let (mut window, _) = press_add_driver(Picked::Cancelled);
    assert!(window.window().settings.user_modules.is_empty());
    assert_eq!(window.window().state.driver_input, None);
    window
        .harness
        .get_by_role_and_label(Role::Button, "Add driver…");
}
