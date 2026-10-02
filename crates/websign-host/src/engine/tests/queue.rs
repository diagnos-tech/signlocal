//! Scenarios 11 and 13: one window, ten waiting, device events.

use websign_devices::monitor::DeviceEvent;
use websign_protocol::{AppMessage, ErrorCode};
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::UiEvent;

use super::rig::Rig;
use crate::engine::EngineEvent;
use crate::ports::KeyCommand;
use crate::testing::fixture;

#[test]
fn eleven_requests_are_accepted_and_the_twelfth_is_busy() {
    let mut rig = Rig::browser();
    for n in 0..11 {
        rig.sign_begin(&format!("r{n}"));
    }
    assert!(rig.messages().is_empty());
    rig.sign_begin("r11");
    let sent = rig.messages();
    assert!(matches!(&sent[..], [AppMessage::Error(e)] if e.code == ErrorCode::Busy));
}

#[test]
fn finishing_the_first_request_puts_the_second_on_screen() {
    let mut rig = Rig::browser();
    for n in 0..11 {
        rig.sign_begin(&format!("r{n}"));
    }
    let commands = rig.h.ui.take();
    let first = Rig::opened(&commands).expect("first opens");
    assert_eq!(first.position, (1, 1), "only the first is open at first");
    assert_eq!(
        commands
            .iter()
            .filter(|c| matches!(c, UiCommand::Open(_)))
            .count(),
        1
    );

    rig.ui(UiEvent::Cancel {
        key: first.key,
        code: ErrorCode::UserCancelled,
    });
    let commands = rig.h.ui.take();
    let second = Rig::opened(&commands).expect("second opens");
    assert_ne!(second.key, first.key);
    assert_eq!(second.position, (1, 10));
}

#[test]
fn requests_are_answered_one_at_a_time_in_order() {
    let mut rig = Rig::browser();
    rig.sign_begin("a");
    rig.sign_begin("b");
    let key_a = Rig::opened(&rig.h.ui.take()).unwrap().key;
    rig.listed(fixture::snapshot());
    rig.ui(UiEvent::Cancel {
        key: key_a,
        code: ErrorCode::UserCancelled,
    });
    let sent = rig.h.outbound.take();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].id.as_str(), "a");
    // The next request lists again when it reaches the front.
    let keys = rig.h.keys.take();
    assert!(matches!(
        keys.last(),
        Some(KeyCommand::List { refresh: false })
    ));
}

#[test]
fn a_queued_request_cancelled_by_its_client_never_reaches_the_window() {
    let mut rig = Rig::browser();
    rig.sign_begin("a");
    rig.sign_begin("b");
    rig.h.ui.take();
    rig.frame(r#""id":"b","type":"cancel""#);
    let sent = rig.h.outbound.take();
    assert_eq!(sent.len(), 1);
    assert_eq!(sent[0].id.as_str(), "b");
    let ui = rig.h.ui.take();
    assert!(
        ui.iter()
            .all(|c| !matches!(c, UiCommand::Open(_) | UiCommand::Finished { .. }))
    );
}

#[test]
fn a_device_event_while_selecting_relists_and_keeps_the_selection() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    let key = Rig::opened(&rig.h.ui.take()).unwrap().key;
    rig.listed(fixture::snapshot());
    rig.h.keys.take();
    rig.h.ui.take();

    rig.engine
        .handle(EngineEvent::Device(DeviceEvent::CardInserted {
            reader: "reader".into(),
            atr: None,
        }));
    assert!(matches!(
        rig.h.keys.take().as_slice(),
        [KeyCommand::Invalidate, KeyCommand::List { refresh: true }]
    ));
    rig.listed(fixture::snapshot());
    let ui = rig.h.ui.take();
    assert!(matches!(&ui[..], [UiCommand::Certificates { key: k, .. }] if *k == key));
    assert!(
        rig.messages().is_empty(),
        "a new site releases nothing by itself"
    );
}

#[test]
fn device_events_are_ignored_when_nothing_is_on_screen() {
    let mut rig = Rig::browser();
    rig.engine
        .handle(EngineEvent::Device(DeviceEvent::ReaderAdded {
            reader: "r".into(),
        }));
    assert!(rig.h.keys.is_empty());
}

#[test]
fn a_removed_token_also_ends_its_sessions() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    rig.h.keys.take();
    rig.engine
        .handle(EngineEvent::Device(DeviceEvent::CardRemoved {
            reader: "r".into(),
        }));
    assert!(matches!(
        rig.h.keys.take().as_slice(),
        [
            KeyCommand::EndSessions,
            KeyCommand::Invalidate,
            KeyCommand::List { refresh: true }
        ]
    ));
}
