use std::cell::Cell;
use std::rc::Rc;

use websign_core::{Fingerprint, HashAlgorithm, SignatureAlgorithm, SourceKind};

use super::*;
use crate::fake::{FakeKeystore, key};
use crate::{Keystore, SourceFailure};

const SHARED: &[u8] = b"certificate seen by the OS and a module";
const MODULE_ONLY: &[u8] = b"certificate only a module sees";

/// A legacy store: PKCS#1 v1.5 only.
fn os_capabilities() -> KeyCapabilities {
    KeyCapabilities::of(&[SignatureAlgorithm::RsaPkcs1v15])
}

struct Setup {
    hub: KeystoreHub,
    lists: Rc<Cell<usize>>,
    ended: Rc<Cell<usize>>,
}

/// A module listed before the OS (the hub must still prefer the OS), a
/// broken source, and a source that failed to open.
fn setup() -> Setup {
    let module = FakeKeystore::new(
        "pkcs11:fake",
        vec![
            key(SHARED, SourceKind::Pkcs11, "module-shared"),
            key(MODULE_ONLY, SourceKind::Pkcs11, "module-only"),
        ],
    );
    let (lists, ended) = (module.lists.clone(), module.ended.clone());
    let os = FakeKeystore {
        capabilities: Some(os_capabilities()),
        ..FakeKeystore::new("os", vec![key(SHARED, SourceKind::System, "os-shared")])
    };
    let broken = FakeKeystore {
        fail_list: true,
        ..FakeKeystore::new("broken", Vec::new())
    };
    let keystores: Vec<Box<dyn Keystore>> = vec![Box::new(module), Box::new(os), Box::new(broken)];
    let opened = Opened {
        keystores,
        failures: vec![SourceFailure {
            source: "pkcs11:missing".to_owned(),
            error: "cannot load".to_owned(),
        }],
    };
    Setup {
        hub: KeystoreHub::with_sources(opened),
        lists,
        ended,
    }
}

fn reference(cert: &[u8], path: usize) -> KeyRef {
    KeyRef {
        fingerprint: Fingerprint::of(cert),
        path,
    }
}

fn signed_by(hub: &mut KeystoreHub, key: KeyRef) -> Result<String, KeystoreError> {
    let digest = [0u8; 32];
    let request = SignRequest {
        hash: HashAlgorithm::Sha256,
        algorithm: SignatureAlgorithm::Ecdsa,
        digest: &digest,
        pin: None,
        parent_window: None,
    };
    hub.sign(key, &request)
        .map(|signature| String::from_utf8_lossy(&signature.bytes).into_owned())
}

#[test]
fn the_listing_is_cached_until_invalidated_and_sources_stay_open() {
    let Setup { mut hub, lists, .. } = setup();
    assert_eq!(hub.inventory().entries.len(), 3);
    hub.inventory();
    assert_eq!(lists.get(), 1);
    hub.invalidate();
    assert_eq!(hub.inventory().entries.len(), 3);
    assert_eq!(lists.get(), 2);
}

#[test]
fn listing_failures_are_reported_once_per_listing() {
    let Setup { mut hub, .. } = setup();
    assert_eq!(hub.inventory().warnings().len(), 2);
    hub.invalidate();
    let warnings = hub.inventory().warnings();
    assert_eq!(
        warnings,
        ["pkcs11:missing: cannot load", "broken: listing failed"]
    );
}

#[test]
fn path_zero_is_the_os_and_alternates_follow() {
    let Setup { mut hub, .. } = setup();
    assert_eq!(
        signed_by(&mut hub, reference(SHARED, 0)).unwrap(),
        "os-shared"
    );
    assert_eq!(
        signed_by(&mut hub, reference(SHARED, 1)).unwrap(),
        "module-shared"
    );
    assert_eq!(
        signed_by(&mut hub, reference(MODULE_ONLY, 0)).unwrap(),
        "module-only"
    );
}

#[test]
fn unknown_keys_and_paths_are_not_found() {
    let Setup { mut hub, .. } = setup();
    for key in [
        reference(SHARED, 2),
        reference(MODULE_ONLY, 1),
        reference(b"other", 0),
    ] {
        assert!(matches!(
            signed_by(&mut hub, key),
            Err(KeystoreError::NotFound)
        ));
        assert!(hub.chain(key).is_empty());
        assert_eq!(hub.pin_state(key), None);
        assert_eq!(hub.capabilities(key), KeyCapabilities::NONE);
    }
}

#[test]
fn capabilities_follow_the_path() {
    let Setup { mut hub, .. } = setup();
    assert_eq!(hub.capabilities(reference(SHARED, 0)), os_capabilities());
    assert_eq!(hub.capabilities(reference(SHARED, 1)), KeyCapabilities::ALL);
}

#[test]
fn chain_and_pin_state_follow_the_path() {
    let Setup { mut hub, .. } = setup();
    assert_eq!(hub.chain(reference(SHARED, 1)), [b"module-shared".to_vec()]);
    assert!(
        hub.pin_state(reference(SHARED, 1))
            .is_some_and(|state| state.unlocked)
    );
}

#[test]
fn end_sessions_reaches_every_source_cached_or_not() {
    let Setup { mut hub, ended, .. } = setup();
    hub.end_sessions();
    assert_eq!(ended.get(), 1);
    hub.inventory();
    hub.end_sessions();
    assert_eq!(ended.get(), 2);
    assert!(
        hub.pin_state(reference(SHARED, 1))
            .is_some_and(|state| !state.unlocked)
    );
}

#[test]
fn a_new_hub_opens_nothing_until_used() {
    let hub = KeystoreHub::new(Options::default());
    assert!(format!("{hub:?}").contains("opened: false"));
}
