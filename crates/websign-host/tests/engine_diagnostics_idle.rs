//! SPEC §8.14 and §8.15: `diagnostics.open`, and the idle exit of a desktop
//! connection.

mod common;

use common::harness::Harness;
use common::{Cert, ORIGIN, wire};
use websign_core::HashAlgorithm;
use websign_host::Control;
use websign_protocol::ErrorCode;
use websign_protocol::messages::DiagnosticsTab;
use websign_ui_model::confirm::UiEvent;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

#[test]
fn diagnostics_open_starts_the_launcher_and_answers_done() {
    let mut h = Harness::native_ready();
    h.send(wire::diagnostics("d1", None));
    let out = h.take();
    assert_eq!(out.diagnostics, [None]);
    assert_eq!(out.kinds(), ["done"]);
    assert_eq!(out.frames[0].id.as_str(), "d1");
    assert!(
        out.ui.is_empty(),
        "the diagnostics window is another process, not ours"
    );
    assert!(out.keys.is_empty());
}

#[test]
fn the_requested_tab_is_passed_to_the_launcher() {
    for (name, tab) in [
        ("browsers", DiagnosticsTab::Browsers),
        ("devices", DiagnosticsTab::Devices),
        ("certificates", DiagnosticsTab::Certificates),
        ("help", DiagnosticsTab::Help),
    ] {
        let mut h = Harness::desktop_ready();
        h.send(wire::diagnostics("d", Some(name)));
        assert_eq!(h.take().diagnostics, [Some(tab)]);
    }
}

#[test]
fn diagnostics_open_does_not_wait_in_the_queue() {
    let mut h = Harness::native_ready();
    h.begin("a", ORIGIN, SHA256);
    h.send(wire::diagnostics("d1", None));
    let out = h.take();
    assert_eq!(out.kinds(), ["done"]);
    assert_eq!(out.diagnostics.len(), 1);
}

#[test]
fn a_launcher_failure_is_reported_instead_of_done() {
    let mut h = Harness::native_ready();
    h.rec.borrow_mut().launcher_fails = true;
    h.send(wire::diagnostics("d1", None));
    let out = h.take();
    assert_eq!(out.only_error(), ("d1".to_owned(), ErrorCode::Internal));
    assert!(!out.errors()[0].1.message.is_empty());
}

#[test]
fn the_window_can_ask_for_diagnostics_too() {
    let p = Cert::p256();
    let mut h = Harness::native_ready();
    h.begin("a", ORIGIN, SHA256);
    h.listed(&[&p]);
    let out = h.ui_out(UiEvent::OpenDiagnostics {
        tab: Some(DiagnosticsTab::Devices),
    });
    assert_eq!(out.diagnostics, [Some(DiagnosticsTab::Devices)]);
    assert!(out.frames.is_empty(), "the caller is not involved");
}

#[test]
fn an_idle_desktop_connection_exits_after_300_seconds() {
    let mut h = Harness::desktop_ready();
    assert_eq!(h.pass(299), Control::Continue);
    assert_eq!(h.pass(1), Control::Exit(0));
}

#[test]
fn a_request_restarts_the_desktop_idle_clock() {
    let mut h = Harness::desktop_ready();
    h.pass(200);
    h.send(wire::status("s", None));
    h.take();
    assert_eq!(
        h.pass(299),
        Control::Continue,
        "300 s counted from the last request"
    );
    assert_eq!(h.pass(1), Control::Exit(0));
}

#[test]
fn a_desktop_connection_with_an_open_request_is_not_idle() {
    let p = Cert::p256();
    let mut h = Harness::desktop_ready();
    h.send(wire::sign_begin("s1", None, "SHA-256"));
    h.take();
    h.listed(&[&p]);
    // The request has its own limits (§4); idleness means "no request open".
    assert_eq!(h.pass(200), Control::Continue);
}

#[test]
fn a_browser_connection_is_never_idle_closed_by_the_host() {
    // The browser owns the port and closes it after 60 s of idleness
    // (limits::EXTENSION_IDLE_CLOSE); the host does not exit by itself.
    let mut h = Harness::native_ready();
    assert_eq!(h.pass(3_600), Control::Continue);
}
