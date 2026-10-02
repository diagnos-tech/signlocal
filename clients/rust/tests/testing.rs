//! The public fake app (feature `testing`), through the public API only.

#![cfg(feature = "testing")]

use websign_client::testing::{FakeApp, sample_certificate};
use websign_client::{
    ClientError, ErrorCode, HashName, SignOptions, SignatureAlgorithmName,
    SignatureAlgorithmName as Alg,
};
use websign_protocol::ClientMessage;
use websign_protocol::types::{FingerprintHex, KeyDescription};

fn digest(len: usize) -> Result<Vec<u8>, String> {
    Ok(vec![9; len])
}

fn rsa() -> websign_protocol::types::Certificate {
    let mut certificate = sample_certificate();
    certificate.fingerprint = FingerprintHex::new("cd".repeat(32)).unwrap();
    certificate.key = KeyDescription::Rsa { bits: 2048 };
    certificate.algorithms = vec![Alg::RsaPkcs1v15];
    certificate
}

#[test]
fn a_default_fake_signs_end_to_end() {
    let app = FakeApp::new();
    let mut client = app.connect().unwrap();
    let signed = client
        .sign(SignOptions::new(HashName::Sha256), |_, _| digest(32))
        .unwrap();
    assert_eq!(signed.algorithm, Alg::Ecdsa);
    // The fake echoes the digest: a stand-in, never a valid signature.
    assert_eq!(signed.signature.as_bytes(), [9; 32]);
    let kinds: Vec<_> = app.requests().iter().map(ClientMessage::kind).collect();
    assert_eq!(kinds, ["hello", "sign.begin", "sign.digest"]);
}

#[test]
fn status_certificates_and_diagnostics_answer() {
    let app = FakeApp::builder().remembered(true).build();
    let mut client = app.connect().unwrap();
    assert!(client.status().unwrap().remembered);
    assert_eq!(client.certificates(&[]).unwrap().len(), 1);
    client.open_diagnostics(None).unwrap();
}

#[test]
fn certificates_are_filtered_and_preselected() {
    let app = FakeApp::builder()
        .certificate(sample_certificate())
        .certificate(rsa())
        .build();
    let mut client = app.connect().unwrap();
    let chosen = client.certificates(&[Alg::RsaPkcs1v15]).unwrap();
    assert_eq!(chosen.len(), 1);
    let options = SignOptions::new(HashName::Sha256).certificate(&chosen[0]);
    let signed = client.sign(options, |_, _| digest(32)).unwrap();
    assert_eq!(signed.algorithm, Alg::RsaPkcs1v15);
    assert!(matches!(
        client.certificates(&[SignatureAlgorithmName::RsaPss]),
        Err(ClientError::App {
            code: ErrorCode::NoCertificates,
            ..
        })
    ));
}

#[test]
fn a_configured_failure_carries_code_and_hint() {
    let app = FakeApp::builder()
        .fail_at_choose(ErrorCode::UserCancelled)
        .build();
    let mut client = app.connect().unwrap();
    let error = client
        .sign(SignOptions::new(HashName::Sha256), |_, _| {
            panic!("prepare must not run")
        })
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::UserCancelled);
    assert!(error.hint().contains("Cancel"));
    assert!(error.docs_url().ends_with("#error-UserCancelled"));
}

#[test]
fn a_failure_at_confirm_happens_after_prepare() {
    let app = FakeApp::builder()
        .fail_at_confirm(ErrorCode::PinLocked)
        .build();
    let mut client = app.connect().unwrap();
    let mut prepared = 0;
    let error = client
        .sign(SignOptions::new(HashName::Sha256), |_, _| {
            prepared += 1;
            digest(32)
        })
        .unwrap_err();
    assert_eq!((prepared, error.code()), (1, ErrorCode::PinLocked));
}

#[test]
fn the_digest_length_rule_is_enforced_by_the_client() {
    let app = FakeApp::new();
    let mut client = app.connect().unwrap();
    let error = client
        .sign(SignOptions::new(HashName::Sha384), |_, _| digest(32))
        .unwrap_err();
    assert_eq!(error.code(), ErrorCode::InvalidRequest);
}

#[test]
fn connections_are_independent_and_dropping_one_is_clean() {
    let app = FakeApp::new();
    drop(app.connect().unwrap());
    let mut second = app.connect().unwrap();
    assert!(second.status().is_ok());
}
