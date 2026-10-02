//! "Still reading {device}" (`docs/ux.md` §4.8), "View in system" (§5.12)
//! and the alternate path's PIN (§4.6, §5.11).

use websign_keystores::{KeyRef, KeystoreError};
use websign_ui_model::certs::{KeyPath, KeySource, PinMode};
use websign_ui_model::confirm::port::Failure;
use websign_ui_model::confirm::{UiCommand, UiEvent};

use super::rig::{Rig, fp};
use crate::ports::{KeyCommand, KeyReply, KeySnapshot};
use crate::testing::fixture;

const TOKEN: &str = "SafeNet eToken 5110";

fn slow() -> KeyReply {
    KeyReply::SlowListing {
        device: Some(TOKEN.to_owned()),
    }
}

fn slow_notices(rig: &Rig) -> Vec<UiCommand> {
    rig.h
        .ui
        .take()
        .into_iter()
        .filter(|command| matches!(command, UiCommand::SlowListing { .. }))
        .collect()
}

#[test]
fn a_slow_listing_names_the_device_until_the_listing_arrives() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    let key = Rig::opened(&rig.h.ui.take()).expect("window opened").key;
    rig.keys(slow());
    assert_eq!(
        slow_notices(&rig),
        [UiCommand::SlowListing {
            key,
            device: Some(TOKEN.to_owned()),
        }]
    );
    rig.listed(fixture::snapshot());
    rig.h.ui.take();
    rig.keys(slow());
    assert_eq!(slow_notices(&rig), [], "the notice crossed its listing");

    rig.ui(UiEvent::Rescan { key });
    rig.keys(slow());
    assert_eq!(slow_notices(&rig).len(), 1, "\"Scan again\" lists anew");
}

#[test]
fn a_slow_listing_without_a_window_says_nothing() {
    let mut rig = Rig::browser();
    rig.keys(slow());
    assert_eq!(slow_notices(&rig), []);
}

#[test]
fn view_in_system_hands_the_listed_der_to_the_window_port() {
    let mut rig = Rig::browser();
    let key = rig.until_ready("1");
    rig.h.outbound.take();
    rig.ui(UiEvent::ViewCertificate {
        key,
        fingerprint: fp(),
    });
    assert_eq!(rig.h.ui.viewed(), [fixture::der()]);

    let unknown = websign_core::Fingerprint::from_bytes([9; 32]);
    rig.ui(UiEvent::ViewCertificate {
        key,
        fingerprint: unknown,
    });
    let stale = websign_ui_model::confirm::port::RequestKey(key.0 + 1);
    rig.ui(UiEvent::ViewCertificate {
        key: stale,
        fingerprint: fp(),
    });
    assert_eq!(rig.h.ui.viewed().len(), 1, "unlisted or stale: nothing");
    assert!(rig.messages().is_empty(), "the caller hears nothing");
}

/// The fixture certificate in the Windows store, with its token driver as
/// the alternate path: the store shows its own PIN dialog, the driver
/// needs ours.
fn store_with_driver() -> KeySnapshot {
    let mut snapshot = fixture::snapshot();
    let candidate = &mut snapshot.candidates[0];
    candidate.alternates = vec![KeyPath {
        source: candidate.source.clone(),
        pin: PinMode::App {
            length: Some((4, 8)),
            count_low: true,
            final_try: false,
            locked: false,
        },
    }];
    candidate.source = KeySource::Windows;
    candidate.pin = PinMode::System;
    snapshot
}

fn sign_command(rig: &Rig) -> (u64, KeyRef, bool) {
    rig.h
        .keys
        .take()
        .into_iter()
        .find_map(|command| match command {
            KeyCommand::Sign { tag, key, pin, .. } => Some((tag, key, pin.is_some())),
            _ => None,
        })
        .expect("a Sign command")
}

fn failures(rig: &Rig) -> Vec<Failure> {
    rig.h
        .ui
        .take()
        .into_iter()
        .filter_map(|command| match command {
            UiCommand::Failed { failure, .. } => Some(failure),
            _ => None,
        })
        .collect()
}

#[test]
fn the_driver_path_signs_with_our_pin_after_the_store_failed() {
    let mut rig = Rig::browser();
    rig.sign_begin("1");
    let key = Rig::opened(&rig.h.ui.take()).expect("window opened").key;
    rig.listed(store_with_driver());
    rig.ui(UiEvent::Continue {
        key,
        fingerprint: fp(),
    });
    rig.answer_chains();
    rig.digest("1", 1, &fixture::DIGEST);
    rig.h.keys.take();
    rig.h.ui.take();
    rig.h.outbound.take();

    rig.press_sign(key, false);
    let (tag, path, _) = sign_command(&rig);
    assert_eq!(path.path, 0);
    rig.keys(KeyReply::Signed {
        tag,
        result: Err(KeystoreError::Native {
            api: "NCryptSignHash",
            code: 0x8010_0001,
            message: "SCARD_F_INTERNAL_ERROR".into(),
        }),
    });
    assert!(matches!(
        failures(&rig).as_slice(),
        [Failure::DriverFailure { driver, alternate: true, .. }] if driver == "Windows"
    ));

    let sign_via_driver = |pin: &str| UiEvent::Sign {
        key,
        fingerprint: fp(),
        via: 1,
        pin: Some(secrecy::SecretString::from(pin.to_owned())),
        remember: false,
    };
    rig.ui(sign_via_driver("0000"));
    let (tag, path, pin) = sign_command(&rig);
    assert_eq!((path.path, pin), (1, true), "the PIN reaches C_Login");
    rig.keys(KeyReply::Signed {
        tag,
        result: Err(KeystoreError::WrongPin),
    });
    assert_eq!(
        failures(&rig),
        [Failure::PinIncorrect {
            count_low: true,
            final_try: false
        }],
        "the driver's PIN flags, not the store's"
    );

    rig.ui(sign_via_driver("1234"));
    let (tag, _, _) = sign_command(&rig);
    rig.keys(KeyReply::Signed {
        tag,
        result: Ok(fixture::signature()),
    });
    assert!(matches!(
        rig.messages().as_slice(),
        [websign_protocol::AppMessage::SignResult(_)]
    ));
}
