//! Starting the app: what `connect` reports when it cannot.

mod common;

use std::path::PathBuf;
use std::time::{Duration, Instant};

use websign_client::{Client, ClientError, ConnectOptions, ErrorCode};

use common::{Fake, QUICK_GRACE, QUICK_HELLO};

#[test]
fn a_missing_executable_is_app_missing() {
    let result = Client::connect_with(ConnectOptions {
        executable: Some(PathBuf::from("/nonexistent/websign")),
        ..ConnectOptions::default()
    });
    assert!(matches!(result, Err(ClientError::AppMissing(_))));
}

#[test]
fn an_app_that_exits_at_once_is_app_missing() {
    let error = Fake::new("early_exit").connect().unwrap_err();
    assert_eq!(error.code(), ErrorCode::AppMissing);
}

#[test]
fn an_error_reply_to_hello_keeps_its_code() {
    let error = Fake::new("hello_error").connect().unwrap_err();
    assert!(matches!(
        error,
        ClientError::App {
            code: ErrorCode::ClientOutdated,
            ..
        }
    ));
}

#[test]
fn a_refusal_at_another_version_than_our_hello_is_a_connection_error() {
    let error = Fake::new("hello_error_wrong_v").connect().unwrap_err();
    assert!(matches!(error, ClientError::Connection(_)), "{error:?}");
}

#[test]
fn an_app_that_only_speaks_newer_versions_is_a_client_problem() {
    let error = Fake::new("newer_app").connect().unwrap_err();
    assert_eq!(error.code(), ErrorCode::ClientOutdated);
}

#[test]
fn a_protocol_version_we_did_not_agree_to_is_a_connection_error() {
    let error = Fake::new("bad_protocol").connect().unwrap_err();
    assert!(matches!(error, ClientError::Connection(_)));
}

#[test]
fn an_unparseable_hello_reply_is_a_connection_error() {
    let error = Fake::new("garbage_hello").connect().unwrap_err();
    assert!(matches!(error, ClientError::Connection(_)));
}

#[test]
fn a_silent_app_times_out_and_is_cleaned_up() {
    let started = Instant::now();
    let error = Fake::new("silent").connect_quickly().unwrap_err();
    assert_eq!(error.code(), ErrorCode::Timeout);
    // The wait for the reply, then the exit grace and the kill: never a hang.
    let elapsed = started.elapsed();
    assert!(elapsed >= QUICK_HELLO, "{elapsed:?}");
    assert!(elapsed < QUICK_HELLO + QUICK_GRACE + Duration::from_secs(3));
}
