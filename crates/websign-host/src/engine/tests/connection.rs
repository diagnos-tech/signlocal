//! Scenarios 1, 2, 9 (disconnect), 14 and 15: the connection itself.

use std::time::Duration;

use websign_protocol::messages::DiagnosticsTab;
use websign_protocol::{AppMessage, ErrorCode};
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::Finish;

use super::rig::{Rig, WEB};
use crate::engine::{Control, EngineEvent};
use crate::launch::{BrowserFamily, BrowserLaunch};
use crate::session::Transport;
use crate::testing::fixture;

fn error_code(message: &AppMessage) -> Option<ErrorCode> {
    match message {
        AppMessage::Error(error) => Some(error.code),
        _ => None,
    }
}

#[test]
fn hello_is_answered_with_the_negotiated_version() {
    let mut rig = Rig::new(Transport::NativeMessaging {
        launch: BrowserLaunch::manual(),
    });
    let control = rig.frame(
        r#""id":"h","type":"hello","client":{"name":"ext","version":"2"},"protocols":{"min":1,"max":1},"browser":{"name":"firefox","version":"130","reason":"startup"}"#,
    );
    assert_eq!(control, Control::Continue);
    let sent = rig.h.outbound.take();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].v, 1);
    assert_eq!(sent[0].id.as_str(), "h");
    assert!(matches!(&sent[0].message, AppMessage::Hello(reply) if reply.protocol == 1));
}

#[test]
fn a_client_that_needs_a_newer_protocol_gets_the_reason_and_the_connection_ends() {
    let mut rig = Rig::new(Transport::NativeMessaging {
        launch: BrowserLaunch::manual(),
    });
    let frame = br#"{"v":3,"id":"h","type":"hello","client":{"name":"ext","version":"9"},"protocols":{"min":2,"max":3},"browser":{"name":"chrome","version":"1","reason":"page"}}"#;
    let control = rig.engine.handle(EngineEvent::Frame(frame.to_vec()));
    assert_eq!(control, Control::Exit(1));
    let sent = rig.h.outbound.take();
    assert_eq!(sent[0].v, 3, "a refusal speaks the version of the hello");
    assert_eq!(error_code(&sent[0].message), Some(ErrorCode::AppOutdated));
}

#[test]
fn a_desktop_hello_with_a_browser_is_refused() {
    let mut rig = Rig::new(super::rig::program_transport());
    let frame = br#"{"v":1,"id":"h","type":"hello","client":{"name":"t","version":"1"},"protocols":{"min":1,"max":1},"browser":{"name":"chrome","version":"1","reason":"page"}}"#;
    assert_eq!(
        rig.engine.handle(EngineEvent::Frame(frame.to_vec())),
        Control::Exit(1)
    );
    assert_eq!(
        error_code(&rig.messages()[0]),
        Some(ErrorCode::InvalidRequest)
    );
}

#[test]
fn status_before_hello_is_an_error_and_closes() {
    let mut rig = Rig::new(Transport::NativeMessaging {
        launch: BrowserLaunch::manual(),
    });
    let control = rig.frame(&format!(r#""id":"1","type":"status",{WEB}"#));
    assert_eq!(control, Control::Exit(1));
    assert_eq!(
        error_code(&rig.messages()[0]),
        Some(ErrorCode::InvalidRequest)
    );
}

#[test]
fn a_second_hello_is_refused_and_the_connection_stays() {
    let mut rig = Rig::browser();
    let control = rig.frame(
        r#""id":"again","type":"hello","client":{"name":"ext","version":"2"},"protocols":{"min":1,"max":1},"browser":{"name":"chrome","version":"1","reason":"page"}"#,
    );
    assert_eq!(control, Control::Continue);
    assert_eq!(
        error_code(&rig.messages()[0]),
        Some(ErrorCode::InvalidRequest)
    );
}

#[test]
fn hello_records_the_browser_connection() {
    let mut rig = Rig::browser();
    let control = rig.frame(&format!(r#""id":"1","type":"status",{WEB}"#));
    assert_eq!(control, Control::Continue);
    let messages = rig.messages();
    assert!(matches!(&messages[0], AppMessage::Status(reply) if !reply.remembered));
}

#[test]
fn transport_rules_are_enforced_on_requests() {
    let mut rig = Rig::browser();
    rig.frame(r#""id":"1","type":"sign.begin","hash":"SHA-256""#);
    let insecure = r#""id":"2","type":"sign.begin","web":{"origin":"http://evil.example","topOrigin":"http://evil.example"},"hash":"SHA-256""#;
    rig.frame(insecure);
    let codes: Vec<_> = rig.messages().iter().filter_map(error_code).collect();
    assert_eq!(
        codes,
        [ErrorCode::InvalidRequest, ErrorCode::InsecureOrigin]
    );
    assert!(rig.h.ui.commands().is_empty());

    let mut program = Rig::program();
    program.frame(&format!(r#""id":"1","type":"choose",{WEB}"#));
    assert_eq!(
        error_code(&program.messages()[0]),
        Some(ErrorCode::InvalidRequest)
    );
}

#[test]
fn a_launch_by_an_unknown_extension_is_refused() {
    let launch = BrowserLaunch {
        origin: "chrome-extension://aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa/".into(),
        family: BrowserFamily::Chromium,
        extension_id: "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".into(),
        parent_window: None,
    };
    let mut rig = Rig::new(Transport::NativeMessaging { launch });
    let control = rig.frame(
        r#""id":"h","type":"hello","client":{"name":"ext","version":"2"},"protocols":{"min":1,"max":1},"browser":{"name":"chrome","version":"1","reason":"page"}"#,
    );
    assert_eq!(control, Control::Exit(1));
    assert_eq!(
        error_code(&rig.messages()[0]),
        Some(ErrorCode::InvalidRequest)
    );
}

#[test]
fn a_launch_by_the_product_extension_is_served() {
    let id = websign_project::chromium_extension_ids()[0].to_owned();
    let launch = BrowserLaunch {
        origin: format!("chrome-extension://{id}/"),
        family: BrowserFamily::Chromium,
        extension_id: id,
        parent_window: None,
    };
    let mut rig = Rig::new(Transport::NativeMessaging { launch });
    let control = rig.frame(
        r#""id":"h","type":"hello","client":{"name":"ext","version":"2"},"protocols":{"min":1,"max":1},"browser":{"name":"chrome","version":"1","reason":"page"}"#,
    );
    assert_eq!(control, Control::Continue);
}

#[test]
fn a_disconnect_tells_the_window_and_sends_nothing() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    let key = Rig::opened(&rig.h.ui.take()).unwrap().key;
    rig.listed(fixture::snapshot());
    rig.h.ui.take();
    rig.h.outbound.take();

    assert_eq!(rig.engine.handle(EngineEvent::Closed), Control::Exit(0));
    assert!(rig.h.outbound.sent().is_empty());
    assert_eq!(
        rig.h.ui.take(),
        [UiCommand::Finished {
            key,
            finish: Finish::SiteCancelled
        }]
    );
}

#[test]
fn a_broken_stream_ends_with_failure() {
    let mut rig = Rig::browser();
    let control = rig.engine.handle(EngineEvent::Broken("truncated".into()));
    assert_eq!(control, Control::Exit(1));
}

#[test]
fn diagnostics_open_starts_the_window_and_answers_done() {
    let mut rig = Rig::browser();
    rig.frame(r#""id":"d","type":"diagnostics.open","tab":"devices""#);
    assert_eq!(rig.h.launcher.opened(), [Some(DiagnosticsTab::Devices)]);
    assert!(matches!(rig.messages()[0], AppMessage::Done(_)));

    rig.h.launcher.set_failing(true);
    rig.frame(r#""id":"e","type":"diagnostics.open""#);
    assert_eq!(error_code(&rig.messages()[0]), Some(ErrorCode::Internal));
}

#[test]
fn a_desktop_client_idle_for_five_minutes_exits() {
    let mut rig = Rig::program();
    assert_eq!(rig.tick_after(Duration::from_secs(299)), Control::Continue);
    assert_eq!(rig.tick_after(Duration::from_secs(2)), Control::Exit(0));
}

#[test]
fn a_browser_connection_is_never_idle_closed_by_the_host() {
    let mut rig = Rig::browser();
    assert_eq!(rig.tick_after(Duration::from_secs(3600)), Control::Continue);
}

#[test]
fn a_process_nobody_talks_to_exits_after_the_hello_limit() {
    let mut rig = Rig::new(Transport::NativeMessaging {
        launch: BrowserLaunch::manual(),
    });
    assert_eq!(rig.tick_after(Duration::from_secs(4)), Control::Continue);
    assert_eq!(rig.tick_after(Duration::from_secs(2)), Control::Exit(0));
}
