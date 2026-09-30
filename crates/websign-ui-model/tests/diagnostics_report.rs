//! SPEC §3.3 and ux §8.7: the exact "Copy diagnostics" text as a whole.

mod common;

use common::*;
use websign_ui_model::diagnostics::report::{ReportInput, render};

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
  reader \"Identiv uTrust 2700 R\" · atr 3B:D5:18:FF:81:91:FE:1F:C3:80:73:C8:21:10:0A · certs 1
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
fn the_ux_example_is_reproduced_byte_for_byte() {
    assert_eq!(render(&example()), GOLDEN);
}

#[test]
fn the_text_ends_with_exactly_one_newline() {
    let text = render(&example());
    assert!(text.ends_with('\n'));
    assert!(!text.ends_with("\n\n"));
}

#[test]
fn rendering_is_deterministic() {
    assert_eq!(render(&example()), render(&example()));
}

#[test]
fn empty_sections_print_none() {
    let text = render(&empty_sections());
    let expected = "\
WebeSign diagnostics v1
app: 1.4.0 (msix, x86_64) · protocol 1 · locale pt-BR · scale 125%
os: Windows 11 23H2 (10.0.22631)
render: wgpu/dx12
browsers:
  none
devices:
  none
pkcs11:
  none
certificates:
  usable 3 (os 3, pkcs11 0, deduplicated 1) · hidden 2 (expired 1, login-only 1)
  kinds: icp-brasil-a3 1, icp-brasil-a1 2 · keys: rsa-2048 3
  expiring<=30d 2
complement: n/a
recent errors (last 20):
  none
";
    assert_eq!(text, expected);
}

#[test]
fn the_complement_line_is_printed_as_given() {
    let text = render(&ReportInput {
        complement: s("1.2.0 (outdated)"),
        ..example()
    });
    assert!(text.contains("\ncomplement: 1.2.0 (outdated)\n"));
}

#[test]
fn nothing_personal_can_appear_because_the_input_has_no_place_for_it() {
    // The report is built only from the fields above; whatever is in them is
    // printed, and everything else about the machine is simply not in them.
    let text = render(&example());
    for forbidden in [
        "serial",
        "fingerprint",
        "digest",
        "cpf",
        "cnpj",
        "@",
        "https://",
    ] {
        assert!(
            !text.to_lowercase().contains(forbidden),
            "the golden text must not mention {forbidden:?}"
        );
    }
}

#[test]
fn every_line_is_plain_english_ascii_apart_from_the_separator_dot() {
    for line in render(&example()).lines() {
        assert!(
            line.chars().all(|c| c.is_ascii() || c == '·'),
            "unexpected character in {line:?}"
        );
    }
}
