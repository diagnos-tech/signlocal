//! An app that breaks the protocol after a good start.

mod common;

use std::time::{Duration, Instant};

use websign_client::{ClientError, ErrorCode, HashName, SignOptions};

use common::{Fake, QUICK_GRACE, connect};

fn assert_connection_error(error: ClientError) {
    assert!(matches!(error, ClientError::Connection(_)), "{error:?}");
}

#[test]
fn a_garbage_frame_is_a_connection_error() {
    let (_fake, mut client) = connect("garbage_after");
    assert_connection_error(client.status().unwrap_err());
}

#[test]
fn unknown_fields_are_refused() {
    let (_fake, mut client) = connect("unknown_field");
    assert_connection_error(client.status().unwrap_err());
}

#[test]
fn a_reply_for_another_request_is_refused() {
    let (_fake, mut client) = connect("wrong_id");
    assert_connection_error(client.status().unwrap_err());
}

#[test]
fn an_oversized_frame_is_refused_without_reading_it() {
    let (_fake, mut client) = connect("oversized");
    assert_connection_error(client.status().unwrap_err());
}

#[test]
fn a_truncated_frame_is_a_connection_error() {
    let (_fake, mut client) = connect("truncated");
    assert_connection_error(client.status().unwrap_err());
}

#[test]
fn exiting_in_the_middle_of_a_signature_is_a_connection_error() {
    let (_fake, mut client) = connect("exit_mid_sign");
    let error = client
        .sign(SignOptions::new(HashName::Sha256), |_, _| Ok(vec![0; 32]))
        .unwrap_err();
    assert_connection_error(error);
}

#[test]
fn a_digest_request_for_another_hash_cancels_with_invalid_request_before_prepare() {
    let (_fake, mut client) = connect("other_hash");
    let mut prepared = false;
    let error = client
        .sign(SignOptions::new(HashName::Sha256), |_, _| {
            prepared = true;
            Ok(vec![0; 32])
        })
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::InvalidRequest, "{error:?}");
    assert!(!prepared);
    // The app ended the cancelled request, so the connection is still good.
    client.status().unwrap();
}

#[test]
fn a_failed_connection_stays_failed() {
    let (_fake, mut client) = connect("garbage_after");
    let _ = client.status();
    assert_connection_error(client.certificates(&[]).unwrap_err());
}

#[test]
fn dropping_kills_an_app_that_ignores_the_closed_pipe() {
    let fake = Fake::new("deaf");
    let client = fake.connect_quickly().unwrap();
    let started = Instant::now();
    drop(client);
    // The grace, then the kill: never a hang.
    let elapsed = started.elapsed();
    assert!(elapsed >= QUICK_GRACE, "{elapsed:?}");
    assert!(elapsed < QUICK_GRACE + Duration::from_secs(3));
}
