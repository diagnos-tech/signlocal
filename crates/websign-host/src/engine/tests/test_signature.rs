//! The setup test: which signatures mark `test_signature_done`.

use websign_keystores::KeystoreError;
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::UiEvent;

use super::rig::Rig;
use crate::ports::KeyReply;
use crate::testing::fixture;

fn site_origin() -> String {
    super::super::test_signature::project_origin().unwrap()
}

fn done(rig: &mut Rig) -> bool {
    rig.engine
        .ports
        .stores
        .settings()
        .get()
        .unwrap()
        .test_signature_done
}

/// Signs from `origin` up to the signature reply `result`.
fn sign_from(rig: &mut Rig, origin: &str, result: Result<Vec<u8>, KeystoreError>) {
    let key = rig.until_ready_from("1", origin);
    rig.h.keys.take();
    rig.press_sign(key, false);
    let tag = rig.sign_tag();
    rig.keys(KeyReply::Signed { tag, result });
}

#[test]
fn a_signature_for_the_project_site_marks_the_test_done() {
    let mut rig = Rig::browser();
    assert!(!done(&mut rig));
    sign_from(&mut rig, &site_origin(), Ok(fixture::signature().to_vec()));
    assert!(done(&mut rig));
}

#[test]
fn a_local_test_page_counts_in_test_builds() {
    let mut rig = Rig::browser();
    sign_from(
        &mut rig,
        "http://localhost:4173",
        Ok(fixture::signature().to_vec()),
    );
    assert!(done(&mut rig));
}

#[test]
fn a_signature_for_another_site_does_not() {
    for origin in [
        "https://app.example",
        "https://diagnos-tech.github.io.evil.example",
        "https://evil.example",
    ] {
        let mut rig = Rig::browser();
        sign_from(&mut rig, origin, Ok(fixture::signature().to_vec()));
        assert!(!done(&mut rig), "{origin}");
    }
}

#[test]
fn a_signature_for_a_program_does_not() {
    let mut rig = Rig::program();
    rig.frame(r#""id":"1","type":"sign.begin","hash":"SHA-256""#);
    let key = Rig::opened(&rig.h.ui.take()).unwrap().key;
    rig.listed(fixture::snapshot());
    rig.ui(UiEvent::Continue {
        key,
        fingerprint: fixture::fingerprint(),
    });
    rig.answer_chains();
    rig.digest("1", 1, &fixture::DIGEST);
    rig.h.keys.take();
    rig.press_sign(key, false);
    let tag = rig.sign_tag();
    rig.keys(KeyReply::Signed {
        tag,
        result: Ok(fixture::signature().to_vec()),
    });
    assert!(!done(&mut rig));
}

#[test]
fn a_failed_signature_does_not_mark_the_test() {
    let mut rig = Rig::browser();
    sign_from(&mut rig, &site_origin(), Err(KeystoreError::WrongPin));
    assert!(!done(&mut rig));
}

#[test]
fn a_cancel_does_not_mark_the_test() {
    let mut rig = Rig::browser();
    let key = rig.until_ready_from("1", &site_origin());
    rig.ui(UiEvent::Cancel {
        key,
        code: ErrorCode::UserCancelled,
    });
    assert!(!done(&mut rig));
}
