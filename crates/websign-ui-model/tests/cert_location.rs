//! SPEC §1.4: "where the certificate is", in lay words.

mod common;

use common::{candidate, driver, token};
use websign_ui_model::certs::{CertCandidate, DeviceLabel, KeySource, Place, location};

fn with(source: KeySource, hardware: Option<bool>, device: Option<DeviceLabel>) -> CertCandidate {
    let mut c = candidate(1, "Ana");
    c.source = source;
    c.hardware = hardware;
    c.device = device;
    c
}

#[test]
fn software_key_in_the_windows_store_is_on_this_computer() {
    let l = location(&with(KeySource::Windows, Some(false), None));
    assert_eq!((l.place, l.via_driver), (Place::Computer, false));
}

#[test]
fn software_key_in_the_macos_keychain_is_on_this_computer() {
    let l = location(&with(KeySource::MacosKeychain, Some(false), None));
    assert_eq!((l.place, l.via_driver), (Place::Computer, false));
}

#[test]
fn a_software_key_of_any_source_is_on_this_computer() {
    let l = location(&with(driver("/usr/lib/softhsm.so"), Some(false), None));
    assert_eq!((l.place, l.via_driver), (Place::Computer, true));
    let l = location(&with(KeySource::MacosToken, Some(false), None));
    assert_eq!(l.place, Place::Computer);
}

#[test]
fn a_named_token_shows_its_name_and_the_driver_flag_follows_the_source() {
    let name = "SafeNet eToken 5110";
    let by_driver = location(&with(driver("eTPKCS11.dll"), Some(true), token(name)));
    assert_eq!(
        by_driver.place,
        Place::Token {
            name: name.to_owned()
        }
    );
    assert!(by_driver.via_driver);

    let by_windows = location(&with(KeySource::Windows, Some(true), token(name)));
    assert_eq!(
        by_windows.place,
        Place::Token {
            name: name.to_owned()
        }
    );
    assert!(!by_windows.via_driver);
}

#[test]
fn a_card_in_a_reader_does_not_name_the_reader() {
    let device = Some(DeviceLabel::CardInReader {
        reader: "Identiv uTrust 2700 R".to_owned(),
    });
    let l = location(&with(KeySource::MacosToken, Some(true), device));
    assert_eq!((l.place, l.via_driver), (Place::CardInReader, false));
}

#[test]
fn hardware_without_a_safe_mapping_is_token_or_card() {
    for (hardware, device) in [
        (Some(true), None),
        (None, None),
        (Some(true), Some(DeviceLabel::Unknown)),
        (None, Some(DeviceLabel::Unknown)),
    ] {
        let l = location(&with(KeySource::Windows, hardware, device.clone()));
        assert_eq!(l.place, Place::UnknownHardware, "{hardware:?} {device:?}");
    }
    assert_eq!(
        location(&with(KeySource::MacosToken, None, None)).place,
        Place::UnknownHardware
    );
}

#[test]
fn only_the_driver_source_is_via_driver() {
    for (source, expected) in [
        (KeySource::Windows, false),
        (KeySource::MacosKeychain, false),
        (KeySource::MacosToken, false),
        (driver("x.so"), true),
    ] {
        let l = location(&with(source.clone(), None, None));
        assert_eq!(l.via_driver, expected, "{source:?}");
    }
}
