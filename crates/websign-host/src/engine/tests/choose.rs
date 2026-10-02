//! Scenario 12 and the `choose` and `status` requests.

use websign_protocol::AppMessage;
use websign_ui_model::confirm::port::{Finish, Mode};
use websign_ui_model::confirm::{UiCommand, UiEvent};

use super::rig::{Rig, WEB, fp};
use crate::ports::{KeyCommand, KeySnapshot};
use crate::testing::fixture;

fn choose(rig: &mut Rig, id: &str) {
    rig.frame(&format!(r#""id":"{id}","type":"choose",{WEB}"#));
}

/// Chooses once with "Remember" ticked so the site is remembered.
fn remember_the_site(rig: &mut Rig) {
    choose(rig, "first");
    let key = Rig::opened(&rig.h.ui.take())
        .expect("a new site gets the window")
        .key;
    rig.listed(fixture::snapshot());
    rig.ui(UiEvent::Choose {
        key,
        fingerprint: fp(),
        remember: true,
    });
    rig.answer_chains();
    assert!(
        matches!(&rig.messages()[..], [AppMessage::ChooseResult(r)] if r.certificates.len() == 1)
    );
    rig.h.ui.take();
    rig.h.keys.take();
}

#[test]
fn a_new_site_chooses_in_the_window_and_gets_only_that_certificate() {
    let mut rig = Rig::browser();
    choose(&mut rig, "1");
    let open = Rig::opened(&rig.h.ui.take()).expect("window");
    assert_eq!(open.mode, Mode::Choose);
    rig.listed(fixture::snapshot());
    assert!(rig.messages().is_empty());
    rig.ui(UiEvent::Choose {
        key: open.key,
        fingerprint: fp(),
        remember: false,
    });
    rig.answer_chains();
    let sent = rig.messages();
    assert!(matches!(&sent[..], [AppMessage::ChooseResult(r)] if r.certificates.len() == 1));
    assert!(rig.h.ui.take().contains(&UiCommand::Finished {
        key: open.key,
        finish: Finish::Chosen
    }));
}

#[test]
fn a_remembered_site_is_answered_without_a_window() {
    let mut rig = Rig::browser();
    remember_the_site(&mut rig);

    choose(&mut rig, "2");
    assert!(rig.h.ui.take().is_empty());
    assert!(matches!(
        rig.h.keys.take().as_slice(),
        [KeyCommand::List { refresh: false }]
    ));
    rig.listed(fixture::snapshot());
    rig.answer_chains();
    assert!(
        matches!(&rig.messages()[..], [AppMessage::ChooseResult(r)] if r.certificates.len() == 1)
    );
    assert!(rig.h.ui.take().is_empty());
}

#[test]
fn status_reports_whether_the_site_is_remembered() {
    let mut rig = Rig::browser();
    rig.frame(&format!(r#""id":"s1","type":"status",{WEB}"#));
    assert!(matches!(&rig.messages()[..], [AppMessage::Status(s)] if !s.remembered));
    remember_the_site(&mut rig);
    rig.frame(&format!(r#""id":"s2","type":"status",{WEB}"#));
    assert!(matches!(&rig.messages()[..], [AppMessage::Status(s)] if s.remembered));
    assert!(rig.h.ui.take().is_empty(), "status never opens a window");
}

#[test]
fn a_remembered_site_whose_certificates_are_gone_gets_the_window() {
    let mut rig = Rig::browser();
    remember_the_site(&mut rig);

    choose(&mut rig, "2");
    rig.h.keys.take();
    rig.listed(KeySnapshot::default());
    assert!(rig.messages().is_empty());
    let commands = rig.h.ui.take();
    let open = Rig::opened(&commands).expect("the window opens like for a new caller");
    assert_eq!(open.mode, Mode::Choose);
    assert!(matches!(
        rig.h.keys.take().as_slice(),
        [KeyCommand::List { refresh: false }]
    ));
    rig.listed(fixture::snapshot());
    rig.ui(UiEvent::Choose {
        key: open.key,
        fingerprint: fp(),
        remember: false,
    });
    rig.answer_chains();
    assert!(matches!(&rig.messages()[..], [AppMessage::ChooseResult(_)]));
}

#[test]
fn a_windowless_choose_does_not_disturb_the_request_on_screen() {
    let mut rig = Rig::browser();
    remember_the_site(&mut rig);
    rig.sign_begin("sign");
    let sign_key = Rig::opened(&rig.h.ui.take()).unwrap().key;
    rig.listed(fixture::snapshot());
    rig.h.outbound.take();
    rig.h.ui.take();

    choose(&mut rig, "quick");
    rig.listed(fixture::snapshot());
    rig.answer_chains();
    let sent = rig.h.outbound.take();
    assert!(sent.iter().any(|e| e.id.as_str() == "quick"));
    let ui = rig.h.ui.take();
    assert!(
        ui.iter()
            .all(|c| !matches!(c, UiCommand::Finished { key, .. } if *key != sign_key))
    );
}
