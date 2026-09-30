//! SPEC §4 table, rows `on_digest` and `Ui(Sign)`.

mod common;

use common::Cert;
use common::effects::{failure, sent_error, sign_command, words};
use common::flows::{KEY, awaiting, digest_message, press_sign, ready};
use websign_core::HashAlgorithm;
use websign_host::flow::Effect;
use websign_host::flow::sign::SignState;
use websign_protocol::ErrorCode;
use websign_ui_model::confirm::UiEvent;
use websign_ui_model::confirm::port::Failure;

const SHA256: HashAlgorithm = HashAlgorithm::Sha256;

#[test]
fn a_stale_digest_changes_nothing() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut flow = common::flows::listed(true, &[&p, &r]);
    let SignState::AwaitingDigest { fingerprint, .. } = flow.state.clone() else {
        panic!("released at once: {:?}", flow.state);
    };
    let other = if fingerprint == p.fingerprint { &r } else { &p };
    flow.on_ui(UiEvent::Selected {
        key: KEY,
        fingerprint: other.fingerprint,
    });
    let before = flow.state.clone();
    assert!(matches!(before, SignState::AwaitingDigest { seq: 2, .. }));
    let effects = flow.on_digest(digest_message(1, common::certs::digest(SHA256)));
    assert!(effects.is_empty());
    assert_eq!(flow.state, before);
}

#[test]
fn a_digest_for_a_sequence_never_asked_ends_the_request() {
    let p = Cert::p256();
    let mut flow = common::flows::listed(false, &[&p]);
    let effects = flow.on_digest(digest_message(1, common::certs::digest(SHA256)));
    assert_eq!(words(&effects), ["send:error", "ui:failed"]);
    assert_eq!(sent_error(&effects), Some(ErrorCode::InvalidRequest));
    assert_eq!(flow.state, SignState::Done);
}

#[test]
fn a_wrong_length_digest_ends_the_request() {
    let p = Cert::p256();
    for len in [0, 31, 33, 48, 64] {
        let mut flow = awaiting(&p);
        let effects = flow.on_digest(digest_message(1, vec![1; len]));
        assert_eq!(words(&effects), ["send:error", "ui:failed"], "len {len}");
        assert_eq!(sent_error(&effects), Some(ErrorCode::InvalidRequest));
        assert!(matches!(failure(&effects), Some(Failure::Internal { .. })));
        assert_eq!(flow.state, SignState::Done);
    }
}

#[test]
fn the_length_follows_the_declared_hash() {
    use websign_protocol::types::HashName;
    for (hash, len) in [
        (HashName::Sha256, 32),
        (HashName::Sha384, 48),
        (HashName::Sha512, 64),
    ] {
        let p = Cert::p256();
        let mut flow =
            websign_host::flow::sign::SignFlow::new(KEY, common::flows::begin(hash), true);
        flow.activate(std::time::Instant::now(), &common::flows::presentation());
        flow.on_listed(&common::snapshot(&[&p]), common::flows::context());
        let effects = flow.on_digest(digest_message(1, vec![7; len]));
        assert_eq!(words(&effects), ["ui:digest_ready"], "{hash:?}");
        assert!(matches!(flow.state, SignState::Ready { seq: 1, .. }));
    }
}

#[test]
fn a_good_digest_makes_the_flow_ready_and_shows_its_code() {
    let p = Cert::p256();
    let mut flow = awaiting(&p);
    let effects = flow.on_digest(digest_message(1, common::certs::digest(SHA256)));
    assert_eq!(words(&effects), ["ui:digest_ready"]);
    assert_eq!(
        flow.state,
        SignState::Ready {
            seq: 1,
            fingerprint: p.fingerprint
        }
    );
}

#[test]
fn sign_in_ready_calls_the_key_store_once() {
    let p = Cert::p256();
    let mut flow = ready(&p);
    let effects = press_sign(&mut flow, &p);
    assert_eq!(words(&effects), ["keys:sign", "ui:signing"]);
    let (tag, digest, via) = sign_command(&effects).expect("sign");
    assert_eq!(digest, common::certs::digest(SHA256));
    assert_eq!(via, 0);
    assert_eq!(
        flow.state,
        SignState::Signing {
            tag,
            fingerprint: p.fingerprint
        }
    );
}

#[test]
fn sign_carries_the_chosen_path_and_the_pin() {
    use websign_ui_model::certs::{KeyPath, KeySource, PinMode};
    let p = Cert::p256();
    let mut snapshot = (*common::snapshot(&[&p])).clone();
    snapshot.candidates[0].alternates = vec![
        KeyPath {
            source: KeySource::MacosKeychain,
            pin: PinMode::System,
        },
        KeyPath {
            source: KeySource::Driver {
                path: "libtoken.so".to_owned(),
            },
            pin: PinMode::Unlocked,
        },
    ];
    let mut flow = common::flows::new_flow(true);
    flow.activate(std::time::Instant::now(), &common::flows::presentation());
    flow.on_listed(&snapshot, common::flows::context());
    flow.on_digest(digest_message(1, common::certs::digest(SHA256)));
    let sign = |via| UiEvent::Sign {
        key: KEY,
        fingerprint: p.fingerprint,
        via,
        pin: Some(secrecy::SecretString::from("1234".to_owned())),
        remember: false,
    };
    assert!(flow.on_ui(sign(3)).is_empty(), "there is no fourth path");
    let effects = flow.on_ui(sign(2));
    let command = effects.iter().find_map(|e| match e {
        Effect::Keys(websign_host::ports::KeyCommand::Sign { key, pin, .. }) => {
            Some((key.path, pin.is_some()))
        }
        _ => None,
    });
    assert_eq!(command, Some((2, true)));
}

#[test]
fn sign_is_ignored_outside_ready_or_for_another_certificate() {
    let (p, r) = (Cert::p256(), Cert::rsa());
    let mut waiting = awaiting(&p);
    assert!(press_sign(&mut waiting, &p).is_empty(), "AwaitingDigest");

    let mut flow = ready(&p);
    assert!(press_sign(&mut flow, &r).is_empty(), "another fingerprint");
    assert!(matches!(flow.state, SignState::Ready { .. }));
}
