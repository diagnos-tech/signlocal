//! "Copy diagnostics" puts exactly the text of `docs/ux.md` §8.7 on the
//! clipboard, and nothing personal from the facts it was built from.

use egui::{Event, Key, Modifiers, OutputCommand};
use egui_kittest::kittest::Queryable as _;

use super::fixture;
use super::support::{Setup, open};
use crate::ui::diagnostics::report;

/// The example of `docs/ux.md` §8.7, verbatim.
const GOLDEN: &str = "\
WebeSign diagnostics v1
app: 1.4.0 (msix, x86_64) · protocol 1 · locale pt-BR · scale 125%
os: Windows 11 23H2 (10.0.22631)
render: wgpu/dx12
browsers:
  chrome 129.0 · extension 1.4.2 · host registered · last ping 2026-09-29T14:02Z
  edge 129.0 · extension not seen · host registered
  firefox 131.0 · extension 1.4.2 · host registered · last ping 2026-09-28T20:40Z
devices:
  usb 0529:0620 safenet-etoken-5110 · certs 0
  reader \"Identiv uTrust 2700 R\" · atr 3B:D5:18:FF:81:91:FE:1F:C3:..:..:..:..:..:.. · certs 1
pkcs11:
  %ProgramFiles%\\OpenSC Project\\OpenSC\\pkcs11\\opensc-pkcs11.dll · loaded · slots 1 · tokens 1
  %USERPROFILE%\\Downloads\\wdpkcs_icp.dll · failed: file not found (user-added)
certificates:
  usable 3 (os 3, pkcs11 0, deduplicated 1) · hidden 2 (expired 1, login-only 1)
  kinds: icp-brasil-a3 1, icp-brasil-a1 2 · keys: rsa-2048 3
  expiring<=30d 2
complement: n/a
recent errors (last 20):
  2026-09-29T14:05Z sign PinIncorrect pkcs11 CKR_PIN_INCORRECT
";

#[test]
fn the_report_is_the_golden_text() {
    let text = report::text(&fixture::facts(), &fixture::environment(), fixture::now());
    assert_eq!(text, GOLDEN);
}

#[test]
fn the_report_carries_no_personal_data() {
    let text = report::text(&fixture::facts(), &fixture::environment(), fixture::now());
    for private in [
        "Ana",
        "ANA",
        "Souza",
        "SOUZA",
        "12345678901",
        "456.789",
        "12.345.678",
        "diagnos.health",
        "ana\\",
        "Users",
    ] {
        assert!(
            !text.contains(private),
            "{private:?} leaked into the report"
        );
    }
}

fn copied(window: &super::support::Window) -> Vec<String> {
    window
        .harness
        .output()
        .platform_output
        .commands
        .iter()
        .filter_map(|command| match command {
            OutputCommand::CopyText(text) => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn the_sidebar_button_copies_the_golden_text() {
    let mut window = open(Setup {
        scale: 1.25,
        ..Setup::default()
    });
    window
        .harness
        .get_by_role_and_label(egui::accesskit::Role::Button, "Copy diagnostics")
        .click();
    window.harness.step();
    assert_eq!(copied(&window), [GOLDEN]);
    window.harness.run();
    window
        .harness
        .get_by_label_contains("Diagnostics copied. No names, ID numbers or sites included.");
}

#[test]
fn the_shortcut_copies_the_golden_text() {
    let mut window = open(Setup {
        scale: 1.25,
        ..Setup::default()
    });
    // One frame with the whole chord: the harness would give each queued
    // event a frame of its own, and the copy would be in an earlier output.
    let modifiers = Modifiers::COMMAND | Modifiers::SHIFT;
    let input = window.harness.input_mut();
    input.events.push(Event::ModifiersChanged(modifiers));
    input.events.push(Event::Key {
        key: Key::C,
        physical_key: None,
        pressed: true,
        repeat: false,
        modifiers,
    });
    window.harness.step();
    assert_eq!(copied(&window), [GOLDEN]);
}
