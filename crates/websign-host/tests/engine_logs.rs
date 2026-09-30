//! T10 and `docs/architecture/security.md` §Logs: no personal data in logs.
//!
//! One test drives every path that logs (success, wrong PIN, garbage
//! signature, cancel, timeout, malformed frames, store failure, desktop
//! caller) with recognizable fixture data and greps everything captured.
//! It is a single test because the logger is process-global.

mod common;

use std::sync::Mutex;

use common::harness::Harness;
use common::{Cert, ORIGIN, wire};
use websign_core::{HashAlgorithm, SignatureAlgorithm};
use websign_host::ports::KeyReply;
use websign_keystores::KeystoreError;
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::UiEvent;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;
const PIN: &str = "739105";

struct Capture(Mutex<Vec<String>>);

static CAPTURE: Capture = Capture(Mutex::new(Vec::new()));

impl log::Log for Capture {
    fn enabled(&self, _: &log::Metadata) -> bool {
        true
    }

    fn log(&self, record: &log::Record) {
        if let Ok(mut lines) = self.0.lock() {
            lines.push(format!(
                "{} {}: {}",
                record.level(),
                record.target(),
                record.args()
            ));
        }
    }

    fn flush(&self) {}
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn scenarios(person: &Cert, p256: &Cert) {
    // Success with a remembered site, a PIN and a chain.
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s1", ORIGIN, person);
    let tag = h.press_sign(key, person, Some(PIN)).signs()[0].tag;
    h.signed_ok(tag, person, SHA256, SignatureAlgorithm::Ecdsa);

    // Wrong PIN, driver failure, garbage signature, locked PIN.
    let mut h = Harness::native_ready();
    let key = h.drive_to_ready("s2", ORIGIN, person);
    let tag = h.press_sign(key, person, Some(PIN)).signs()[0].tag;
    h.signed_err(tag, KeystoreError::WrongPin);
    let tag = h.press_sign(key, person, Some(PIN)).signs()[0].tag;
    h.signed_err(
        tag,
        KeystoreError::Native {
            api: "C_Sign",
            code: 0x30,
            message: "device error".into(),
        },
    );
    let tag = h.press_sign(key, person, Some(PIN)).signs()[0].tag;
    h.keys(KeyReply::Signed {
        tag,
        result: Ok(vec![9; 64]),
    });
    let tag = h.press_sign(key, person, Some(PIN)).signs()[0].tag;
    h.signed_err(tag, KeystoreError::PinLocked);
    h.ui(UiEvent::Cancel {
        key,
        code: ErrorCode::PinLocked,
    });

    // New site: continue, cancel by the page; choose; timeout; disconnect.
    let mut h = Harness::native_ready();
    let key = h.begin("s3", ORIGIN, SHA256).opens()[0].key;
    h.listed(&[person, p256]);
    h.ui(UiEvent::Continue {
        key,
        fingerprint: person.fingerprint,
    });
    h.send(wire::digest("s3", 1, &[1; 31]));
    h.send(wire::choose("c1", Some(ORIGIN)));
    h.pass(400);

    let mut h = Harness::native_ready();
    h.begin("s4", ORIGIN, SHA256);
    h.send(wire::cancel("s4"));
    h.send(wire::sign_begin(
        "s5",
        Some("http://evil.example"),
        "SHA-256",
    ));
    h.send(b"{\"v\":1,\"id\":\"x\",\"type\":\"sign.begin\",\"hash\":\"SHA-1\"}".to_vec());
    h.state.borrow_mut().failing = true;
    h.send(wire::status("st", Some(ORIGIN)));
    h.begin("s6", ORIGIN, SHA256);
    h.listed(&[person]);
    h.engine.handle(websign_host::EngineEvent::Broken(
        "framing violated".to_owned(),
    ));

    // A desktop program: its path and product name are identifying too.
    let mut h = Harness::desktop_ready();
    h.send(wire::sign_begin("d1", None, "SHA-256"));
    h.listed(&[person]);
    h.engine.handle(websign_host::EngineEvent::Closed);
}

#[test]
fn logs_carry_steps_and_codes_never_personal_data() {
    log::set_logger(&CAPTURE).expect("first logger in this process");
    log::set_max_level(log::LevelFilter::Trace);

    let person = Cert::person();
    let p256 = Cert::p256();
    scenarios(&person, &p256);

    let lines = CAPTURE.0.lock().expect("lock").clone();
    let log = lines.join("\n");
    let digest = common::certs::digest(SHA256);
    let signature = person.signature(SHA256, SignatureAlgorithm::Ecdsa);

    let forbidden: Vec<(&str, String)> = vec![
        ("holder name", "ANA BEATRIZ".to_owned()),
        ("holder name, any case", "Ana Beatriz".to_owned()),
        ("CPF", "12345678901".to_owned()),
        ("display name", person.info.display_name()),
        ("fingerprint", person.hex()),
        ("other fingerprint", p256.hex()),
        ("certificate serial", person.info.serial_hex.clone()),
        ("digest hex", hex(&digest)),
        ("digest hex upper", hex(&digest).to_uppercase()),
        ("digest base64", websign_protocol::base64::encode(&digest)),
        ("verification code", "4A4A 4EF2".to_owned()),
        ("signature hex", hex(&signature)),
        (
            "signature base64",
            websign_protocol::base64::encode(&signature),
        ),
        (
            "certificate base64",
            websign_protocol::base64::encode(&person.der[..60]),
        ),
        ("PIN", PIN.to_owned()),
        ("origin", "app.example.com".to_owned()),
        ("evil origin", "evil.example".to_owned()),
        ("desktop program path", "/opt/tools/invoicer".to_owned()),
        ("desktop product name", "Invoicer".to_owned()),
    ];
    for (what, needle) in forbidden {
        assert!(
            !needle.is_empty() && !log.contains(&needle),
            "{what} ({needle}) appeared in the log:\n{log}"
        );
    }
}
