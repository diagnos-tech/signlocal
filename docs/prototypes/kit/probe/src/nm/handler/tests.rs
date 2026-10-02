use std::cell::RefCell;
use std::rc::Rc;

use probe_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm};
use serde_json::{Value, json};

use super::*;
use crate::nm::backend::{CertificateList, CertificateSummary, SignedDigest};
use crate::nm::base64;
use crate::nm::launch::BrowserFamily;

const FINGERPRINT: &str = "00112233445566778899aabbccddeeff00112233445566778899aabbccddeeff";

/// What the backend was asked to sign.
struct SignCall {
    fingerprint: Fingerprint,
    hash: HashAlgorithm,
    algorithm: SignatureAlgorithm,
    digest: Vec<u8>,
    parent_window: Option<isize>,
}

#[derive(Default)]
struct Calls {
    lists: usize,
    signs: Vec<SignCall>,
}

struct FakeBackend {
    calls: Rc<RefCell<Calls>>,
    sign_error: Option<ProtocolError>,
}

impl Backend for FakeBackend {
    fn certificates(&mut self) -> Result<CertificateList, ProtocolError> {
        self.calls.borrow_mut().lists += 1;
        Ok(CertificateList {
            certificates: vec![CertificateSummary {
                fingerprint: FINGERPRINT.to_owned(),
                display_name: "Test Holder".to_owned(),
                kind: "certificate".to_owned(),
                level: None,
                key: "EC P-256".to_owned(),
                origin: "fake".to_owned(),
                provider: "fake token".to_owned(),
                hardware: Some(true),
                pin: "app",
                can_sign: true,
                algorithms: vec!["ECDSA"],
                not_before: 0,
                not_after: 1,
                paths: 1,
            }],
            warnings: vec!["pkcs11:x: could not load".to_owned()],
        })
    }

    fn sign(&mut self, job: &SignJob) -> Result<SignedDigest, ProtocolError> {
        self.calls.borrow_mut().signs.push(SignCall {
            fingerprint: job.fingerprint,
            hash: job.hash,
            algorithm: job.algorithm,
            digest: job.digest.clone(),
            parent_window: job.parent_window,
        });
        match &self.sign_error {
            Some(error) => Err(error.clone()),
            None => Ok(SignedDigest {
                signature: vec![1, 2, 3, 4],
                api: "C_Sign",
                elapsed_ms: 7,
                verified: true,
            }),
        }
    }
}

fn launch() -> BrowserLaunch {
    BrowserLaunch {
        origin: "chrome-extension://nhnkdpljdgjflbflkhnkmfmcmodboeii/".to_owned(),
        family: BrowserFamily::Chromium,
        extension_id: "nhnkdpljdgjflbflkhnkmfmcmodboeii".to_owned(),
        parent_window: Some(42),
    }
}

fn handler(sign_error: Option<ProtocolError>) -> (Handler<FakeBackend>, Rc<RefCell<Calls>>) {
    let calls = Rc::new(RefCell::new(Calls::default()));
    let backend = FakeBackend {
        calls: Rc::clone(&calls),
        sign_error,
    };
    (Handler::new(launch(), backend, HostLog::disabled()), calls)
}

fn ask(handler: &mut Handler<FakeBackend>, request: Value) -> Reply {
    handler.handle(request.to_string().as_bytes())
}

fn sign_request(hash: &str, algorithm: &str, digest_len: usize) -> Value {
    json!({
        "v": 1, "id": "s1", "type": "sign",
        "fingerprint": FINGERPRINT, "hash": hash, "algorithm": algorithm,
        "digest": base64::encode(&vec![7u8; digest_len]),
    })
}

fn error_code(reply: &Reply) -> &str {
    reply.message["error"]["code"].as_str().unwrap()
}

#[test]
fn ping_reports_app_platform_and_how_it_was_launched() {
    let (mut handler, _) = handler(None);
    let reply = ask(&mut handler, json!({ "v": 1, "id": "p1", "type": "ping" }));
    let m = &reply.message;
    assert_eq!(reply.outcome, "ok");
    assert_eq!(
        (m["v"].clone(), m["id"].clone(), m["ok"].clone()),
        (json!(1), json!("p1"), json!(true))
    );
    assert_eq!(m["type"], "pong");
    assert_eq!(m["app"]["name"], env!("CARGO_PKG_NAME"));
    assert_eq!(m["app"]["version"], env!("CARGO_PKG_VERSION"));
    assert_eq!(m["os"], std::env::consts::OS);
    assert_eq!(m["arch"], std::env::consts::ARCH);
    assert_eq!(m["launch"]["family"], "chromium");
    assert_eq!(
        m["launch"]["extension_id"],
        "nhnkdpljdgjflbflkhnkmfmcmodboeii"
    );
}

#[test]
fn ping_never_loads_a_key_source() {
    let (mut handler, calls) = handler(None);
    ask(&mut handler, json!({ "v": 1, "id": "p", "type": "ping" }));
    assert_eq!(calls.borrow().lists, 0);
}

#[test]
fn other_versions_are_refused_before_anything_runs() {
    let (mut handler, calls) = handler(None);
    let reply = ask(&mut handler, json!({ "v": 2, "id": "x", "type": "list" }));
    assert_eq!(error_code(&reply), "unsupported_version");
    assert_eq!(reply.message["id"], "x");
    assert_eq!(reply.outcome, "unsupported_version");
    assert_eq!(calls.borrow().lists, 0);
}

#[test]
fn garbage_gets_a_bad_request_and_the_host_keeps_going() {
    let (mut handler, _) = handler(None);
    let reply = handler.handle(b"{ not json");
    assert_eq!(error_code(&reply), "bad_request");
    assert_eq!(reply.message["id"], Value::Null);
    let reply = ask(
        &mut handler,
        json!({ "v": 1, "id": "again", "type": "ping" }),
    );
    assert_eq!(reply.outcome, "ok");
}

#[test]
fn list_returns_certificates_and_warnings() {
    let (mut handler, calls) = handler(None);
    let reply = ask(&mut handler, json!({ "v": 1, "id": "l", "type": "list" }));
    assert_eq!(reply.message["type"], "certificates");
    assert_eq!(reply.message["certificates"][0]["fingerprint"], FINGERPRINT);
    assert_eq!(reply.message["certificates"][0]["pin"], "app");
    assert_eq!(reply.message["warnings"][0], "pkcs11:x: could not load");
    assert_eq!(calls.borrow().lists, 1);
}

#[test]
fn sign_passes_a_validated_job_to_the_backend() {
    let (mut handler, calls) = handler(None);
    let reply = ask(&mut handler, sign_request("SHA-256", "ECDSA", 32));
    assert_eq!(reply.outcome, "ok");
    assert_eq!(reply.message["type"], "signature");
    assert_eq!(reply.message["signature"], base64::encode(&[1, 2, 3, 4]));
    assert_eq!(reply.message["api"], "C_Sign");
    assert_eq!(reply.message["verified"], true);

    let calls = calls.borrow();
    let call = &calls.signs[0];
    assert_eq!(call.fingerprint.to_hex(), FINGERPRINT);
    assert_eq!(call.hash, HashAlgorithm::Sha256);
    assert_eq!(call.algorithm, SignatureAlgorithm::Ecdsa);
    assert_eq!(call.digest, vec![7u8; 32]);
    assert_eq!(
        call.parent_window,
        Some(42),
        "the browser window owns the dialogs"
    );
}

#[test]
fn every_hash_accepts_exactly_its_own_digest_length() {
    for (hash, len) in [("SHA-256", 32), ("SHA-384", 48), ("SHA-512", 64)] {
        let (mut handler, calls) = handler(None);
        assert_eq!(
            ask(&mut handler, sign_request(hash, "ECDSA", len)).outcome,
            "ok"
        );
        for wrong in [0, len - 1, len + 1, 32 + 48 + 64] {
            if wrong == len {
                continue;
            }
            let reply = ask(&mut handler, sign_request(hash, "ECDSA", wrong));
            assert_eq!(
                error_code(&reply),
                "digest_length",
                "{hash} with {wrong} bytes"
            );
        }
        assert_eq!(
            calls.borrow().signs.len(),
            1,
            "wrong sizes never reach the key"
        );
    }
}

#[test]
fn sign_rejects_unknown_algorithms_and_bad_encodings_without_calling_the_backend() {
    let (mut handler, calls) = handler(None);
    let mut cases = vec![
        sign_request("SHA-1", "ECDSA", 20),
        sign_request("MD5", "ECDSA", 16),
        sign_request("SHA-256", "HMAC", 32),
    ];
    let mut bad_digest = sign_request("SHA-256", "ECDSA", 32);
    bad_digest["digest"] = json!("***not base64***");
    cases.push(bad_digest);
    let mut bad_fingerprint = sign_request("SHA-256", "ECDSA", 32);
    bad_fingerprint["fingerprint"] = json!("abc");
    cases.push(bad_fingerprint);
    for case in cases {
        assert_eq!(
            error_code(&ask(&mut handler, case.clone())),
            "bad_request",
            "{case}"
        );
    }
    assert!(calls.borrow().signs.is_empty());
}

#[test]
fn backend_errors_keep_their_stable_code() {
    for code in [
        ErrorCode::NotFound,
        ErrorCode::Cancelled,
        ErrorCode::WrongPin,
        ErrorCode::PinRequired,
        ErrorCode::Internal,
    ] {
        let (mut handler, _) = handler(Some(ProtocolError::new(code, "from the key source")));
        let reply = ask(&mut handler, sign_request("SHA-256", "ECDSA", 32));
        assert_eq!(error_code(&reply), code.as_str());
        assert_eq!(reply.message["error"]["message"], "from the key source");
        assert_eq!(reply.message["id"], "s1");
    }
}

#[test]
fn refusals_have_no_id() {
    let reply = refuse(&ProtocolError::new(ErrorCode::BadRequest, "too big"));
    assert_eq!(reply.message["id"], Value::Null);
    assert_eq!(reply.outcome, "bad_request");
}
