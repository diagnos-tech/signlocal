use std::cell::RefCell;
use std::panic::catch_unwind;
use std::path::PathBuf;
use std::rc::Rc;
use std::time::Duration;

use probe_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm, SourceKind};

use super::fixture::{CERT_DER, SIGNATURE, digest};
use super::sources::Loaded;
use super::*;
use crate::keystores::{FoundKey, Keystore, Opened, Signature};

/// What a fake key source saw when asked to sign.
#[derive(Debug, Default)]
struct Seen {
    signers: Vec<String>,
    pins: Vec<Option<String>>,
    parent_windows: Vec<Option<isize>>,
}

struct FakeKeystore {
    name: &'static str,
    kind: SourceKind,
    pin: PinPrompt,
    signature: Vec<u8>,
    panics: bool,
    seen: Rc<RefCell<Seen>>,
}

impl FakeKeystore {
    fn new(name: &'static str, kind: SourceKind, seen: &Rc<RefCell<Seen>>) -> Self {
        Self {
            name,
            kind,
            pin: PinPrompt::System,
            signature: SIGNATURE.to_vec(),
            panics: false,
            seen: Rc::clone(seen),
        }
    }
}

impl Keystore for FakeKeystore {
    fn name(&self) -> String {
        self.name.to_owned()
    }

    fn list(&mut self) -> Result<Vec<FoundKey>, KeystoreError> {
        Ok(vec![FoundKey {
            cert_der: CERT_DER.to_vec(),
            keystore: self.name.to_owned(),
            kind: self.kind,
            provider: "fake token".to_owned(),
            hardware: Some(true),
            locator: "1".to_owned(),
            pin: self.pin,
        }])
    }

    fn sign(
        &mut self,
        _key: &FoundKey,
        request: &SignRequest<'_>,
    ) -> Result<Signature, KeystoreError> {
        if self.panics {
            panic!("driver exploded");
        }
        use secrecy::ExposeSecret;
        let mut seen = self.seen.borrow_mut();
        seen.signers.push(self.name.to_owned());
        seen.pins
            .push(request.pin.map(|pin| pin.expose_secret().to_owned()));
        seen.parent_windows.push(request.parent_window);
        Ok(Signature {
            bytes: self.signature.clone(),
            api: "C_Sign",
            elapsed: Duration::from_millis(3),
        })
    }
}

fn backend(stores: Vec<FakeKeystore>, pin: Option<&str>) -> KeystoreBackend {
    let opened = Opened {
        keystores: stores
            .into_iter()
            .map(|store| Box::new(store) as Box<dyn Keystore>)
            .collect(),
        failures: Vec::new(),
    };
    KeystoreBackend {
        state: State::Loaded(Loaded::index(opened)),
        pin: pin.map(|pin| SecretString::from(pin.to_owned())),
        options: Options::default(),
    }
}

fn fingerprint() -> Fingerprint {
    Fingerprint::of(&CERT_DER)
}

fn job(algorithm: SignatureAlgorithm) -> SignJob {
    SignJob {
        fingerprint: fingerprint(),
        hash: HashAlgorithm::Sha256,
        algorithm,
        digest: digest(),
        parent_window: Some(42),
    }
}

fn seen() -> Rc<RefCell<Seen>> {
    Rc::default()
}

#[test]
fn lists_a_certificate_with_what_the_extension_needs() {
    let seen = seen();
    let mut backend = backend(
        vec![FakeKeystore::new("pkcs11:fake", SourceKind::Pkcs11, &seen)],
        None,
    );
    let list = backend.certificates().unwrap();
    let cert = &list.certificates[0];
    assert_eq!(list.certificates.len(), 1);
    assert_eq!(cert.fingerprint, fingerprint().to_hex());
    assert_eq!(cert.display_name, "WebeSign Test Holder");
    assert_eq!(cert.kind, "certificate");
    assert_eq!(cert.key, "EC P-256");
    assert_eq!(cert.origin, "pkcs11:fake");
    assert_eq!(cert.provider, "fake token");
    assert_eq!(cert.hardware, Some(true));
    assert_eq!(cert.pin, "system");
    assert!(cert.can_sign);
    assert_eq!(cert.algorithms, ["ECDSA"]);
    assert_eq!(cert.paths, 1);
}

#[test]
fn the_same_certificate_from_two_sources_is_listed_once_and_signed_by_the_os() {
    let seen = seen();
    let mut backend = backend(
        vec![
            FakeKeystore::new("windows", SourceKind::System, &seen),
            FakeKeystore::new("pkcs11:fake", SourceKind::Pkcs11, &seen),
        ],
        None,
    );
    let list = backend.certificates().unwrap();
    assert_eq!(list.certificates.len(), 1);
    assert_eq!(list.certificates[0].paths, 2);
    assert_eq!(list.certificates[0].origin, "windows");

    backend.sign(&job(SignatureAlgorithm::Ecdsa)).unwrap();
    assert_eq!(seen.borrow().signers, ["windows"]);
}

#[test]
fn a_valid_signature_is_returned_verified() {
    let seen = seen();
    let mut backend = backend(
        vec![FakeKeystore::new("windows", SourceKind::System, &seen)],
        None,
    );
    let signed = backend.sign(&job(SignatureAlgorithm::Ecdsa)).unwrap();
    assert_eq!(signed.signature, SIGNATURE);
    assert_eq!(signed.api, "C_Sign");
    assert!(signed.verified);
    assert_eq!(seen.borrow().parent_windows, [Some(42)]);
    assert_eq!(seen.borrow().pins, [None], "OS keys never receive a PIN");
}

#[test]
fn a_signature_that_does_not_verify_is_flagged_not_hidden() {
    let seen = seen();
    let mut store = FakeKeystore::new("windows", SourceKind::System, &seen);
    store.signature = vec![1; 64];
    let signed = backend(vec![store], None)
        .sign(&job(SignatureAlgorithm::Ecdsa))
        .unwrap();
    assert!(!signed.verified);
}

#[test]
fn an_unknown_fingerprint_is_not_found() {
    let seen = seen();
    let mut backend = backend(
        vec![FakeKeystore::new("windows", SourceKind::System, &seen)],
        None,
    );
    let mut request = job(SignatureAlgorithm::Ecdsa);
    request.fingerprint = Fingerprint::from_bytes([9; 32]);
    let error = backend.sign(&request).unwrap_err();
    assert_eq!(error.code, ErrorCode::NotFound);
    assert!(seen.borrow().signers.is_empty());
}

#[test]
fn an_algorithm_the_key_cannot_do_is_unsupported() {
    let seen = seen();
    let mut backend = backend(
        vec![FakeKeystore::new("windows", SourceKind::System, &seen)],
        None,
    );
    let error = backend.sign(&job(SignatureAlgorithm::RsaPss)).unwrap_err();
    assert_eq!(error.code, ErrorCode::Unsupported);
    assert!(seen.borrow().signers.is_empty(), "the key is never asked");
}

#[test]
fn keys_with_an_app_pin_get_the_configured_pin() {
    let seen = seen();
    let mut store = FakeKeystore::new("pkcs11:fake", SourceKind::Pkcs11, &seen);
    store.pin = PinPrompt::App {
        protected_path: false,
    };
    backend(vec![store], Some("1234"))
        .sign(&job(SignatureAlgorithm::Ecdsa))
        .unwrap();
    assert_eq!(seen.borrow().pins, [Some("1234".to_owned())]);
}

#[test]
fn keys_with_an_app_pin_and_no_pin_configured_ask_for_one_without_signing() {
    let seen = seen();
    let mut store = FakeKeystore::new("pkcs11:fake", SourceKind::Pkcs11, &seen);
    store.pin = PinPrompt::App {
        protected_path: false,
    };
    let error = backend(vec![store], None)
        .sign(&job(SignatureAlgorithm::Ecdsa))
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::PinRequired);
    assert!(seen.borrow().signers.is_empty());
}

#[test]
fn a_pin_pad_key_needs_no_pin_from_us() {
    let seen = seen();
    let mut store = FakeKeystore::new("pkcs11:fake", SourceKind::Pkcs11, &seen);
    store.pin = PinPrompt::App {
        protected_path: true,
    };
    backend(vec![store], None)
        .sign(&job(SignatureAlgorithm::Ecdsa))
        .unwrap();
    assert_eq!(seen.borrow().pins, [None]);
}

#[test]
fn a_crashing_key_source_is_an_internal_error_not_a_dead_host() {
    let seen = seen();
    let mut store = FakeKeystore::new("windows", SourceKind::System, &seen);
    store.panics = true;
    let error = backend(vec![store], None)
        .sign(&job(SignatureAlgorithm::Ecdsa))
        .unwrap_err();
    assert_eq!(error.code, ErrorCode::Internal);
    assert!(
        error.message.contains("driver exploded"),
        "{}",
        error.message
    );
}

#[test]
fn keystore_errors_map_to_stable_codes() {
    let cases = [
        (KeystoreError::WrongPin, ErrorCode::WrongPin),
        (KeystoreError::PinLocked, ErrorCode::PinLocked),
        (KeystoreError::PinRequired, ErrorCode::PinRequired),
        (KeystoreError::Cancelled, ErrorCode::Cancelled),
        (KeystoreError::NotFound, ErrorCode::NotFound),
        (
            KeystoreError::Unsupported("x".into()),
            ErrorCode::Unsupported,
        ),
        (KeystoreError::Other("x".into()), ErrorCode::Internal),
        (
            KeystoreError::Native {
                api: "C_Sign",
                code: 5,
                message: "boom".into(),
            },
            ErrorCode::Internal,
        ),
    ];
    for (error, code) in cases {
        assert_eq!(map_error(error).code, code);
    }
}

#[test]
fn panics_become_readable_messages() {
    let literal = catch_unwind(|| panic!("static text")).unwrap_err();
    assert_eq!(panic_message(&*literal), "static text");
    let formatted = catch_unwind(|| panic!("code {}", 7)).unwrap_err();
    assert_eq!(panic_message(&*formatted), "code 7");
}

#[test]
fn extra_modules_come_from_a_path_style_list() {
    let list = std::env::join_paths(["/opt/softhsm/libsofthsm2.so", "/usr/lib/other.so"]).unwrap();
    let options = options_with_modules(Some(list));
    assert_eq!(
        options.extra_modules,
        [
            PathBuf::from("/opt/softhsm/libsofthsm2.so"),
            PathBuf::from("/usr/lib/other.so")
        ]
    );
    assert!(options_with_modules(None).extra_modules.is_empty());
}
