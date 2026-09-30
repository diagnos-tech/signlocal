//! Well-behaved conversations with the fake app.

mod common;

use websign_client::{ClientError, HashName, SignOptions};
use websign_protocol::ErrorCode;
use websign_protocol::messages::DiagnosticsTab;

use common::connect;

#[test]
fn status_reports_the_app() {
    let (_fake, mut client) = connect("ok");
    let status = client.status().unwrap();
    assert_eq!(status.app.version, "1.0.0");
    assert!(status.remembered);
}

#[test]
fn certificates_returns_the_chosen_ones() {
    let (_fake, mut client) = connect("ok");
    let certificates = client.certificates(None).unwrap();
    assert_eq!(certificates.len(), 1);
    assert_eq!(certificates[0].display_name, "Test Holder");
    // Decoded bytes, not Base64 text.
    assert_eq!(certificates[0].der.as_bytes(), [0, 1, 2]);
    assert_eq!(certificates[0].chain[0].as_bytes(), [3, 4]);
}

#[test]
fn diagnostics_opens() {
    let (_fake, mut client) = connect("ok");
    client.open_diagnostics(Some(DiagnosticsTab::Help)).unwrap();
}

#[test]
fn sign_hands_the_certificate_to_prepare_and_returns_the_result() {
    let (_fake, mut client) = connect("ok");
    let mut seen = Vec::new();
    let result = client
        .sign(
            SignOptions::new(HashName::Sha256),
            |certificate, algorithm| {
                seen.push((certificate.display_name.clone(), algorithm));
                Ok(vec![7u8; 32])
            },
        )
        .unwrap();
    assert_eq!(seen.len(), 1);
    assert_eq!(seen[0].0, "Test Holder");
    assert_eq!(result.signature.as_bytes(), [7u8; 32]);
}

#[test]
fn the_connection_serves_several_requests() {
    let (_fake, mut client) = connect("ok");
    client.status().unwrap();
    client.certificates(None).unwrap();
    client
        .sign(SignOptions::new(HashName::Sha384), |_, _| Ok(vec![1; 48]))
        .unwrap();
    client.status().unwrap();
}

#[test]
fn switching_certificate_runs_prepare_again_and_signs_the_last_digest() {
    let (_fake, mut client) = connect("switch");
    let mut calls = 0u8;
    let result = client
        .sign(SignOptions::new(HashName::Sha256), |_, _| {
            calls += 1;
            Ok(vec![calls; 32])
        })
        .unwrap();
    assert_eq!(calls, 2);
    assert_eq!(result.signature.as_bytes(), [2u8; 32]);
}

#[test]
fn prepare_failure_cancels_and_reports_the_reason() {
    let (_fake, mut client) = connect("cancel");
    let error = client
        .sign(SignOptions::new(HashName::Sha256), |_, _| {
            Err("no document".into())
        })
        .unwrap_err();
    assert!(matches!(&error, ClientError::Prepare(why) if why == "no document"));
    assert_eq!(error.code(), ErrorCode::Aborted);
}

#[test]
fn a_digest_of_the_wrong_length_cancels_with_invalid_request() {
    let (_fake, mut client) = connect("cancel");
    let error = client
        .sign(SignOptions::new(HashName::Sha256), |_, _| Ok(vec![0; 3]))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::InvalidRequest);
}

#[test]
fn the_connection_survives_a_cancelled_request() {
    let (_fake, mut client) = connect("cancel");
    let _ = client.sign(SignOptions::new(HashName::Sha256), |_, _| Err("x".into()));
    client.status().unwrap();
}

#[test]
fn app_errors_keep_their_code() {
    let (_fake, mut client) = connect("user_cancel");
    let error = client
        .sign(SignOptions::new(HashName::Sha256), |_, _| Ok(vec![0; 32]))
        .unwrap_err();
    assert!(matches!(
        error,
        ClientError::App {
            code: ErrorCode::UserCancelled,
            ..
        }
    ));
}

#[test]
fn the_public_types_can_cross_threads() {
    fn send<T: Send>() {}
    fn send_sync<T: Send + Sync + 'static>() {}
    send::<websign_client::Client>();
    send_sync::<ClientError>();
    send_sync::<websign_client::ConnectOptions>();
    send_sync::<SignOptions>();
}
