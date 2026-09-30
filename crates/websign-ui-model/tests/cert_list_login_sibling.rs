//! SPEC §1.5: a login certificate next to a signing sibling of the same
//! holder on the same device is hidden (`LoginSibling`).

mod common;

use common::*;
use websign_core::{KeyUsage, PublicKeyKind};
use websign_ui_model::certs::{CertCandidate, DeviceLabel, HiddenReason, build_cert_list};

/// A card's signing certificate and its login sibling on the given devices.
fn on_devices(
    signing_device: Option<DeviceLabel>,
    login_device: Option<DeviceLabel>,
) -> [CertCandidate; 2] {
    let mut signing = candidate(1, "Joao Pereira");
    signing.device = signing_device;
    let mut login = with_info(candidate(2, "Joao Pereira"), |i| {
        i.key_usage = Some(KeyUsage {
            digital_signature: true,
            ..KeyUsage::default()
        })
    });
    login.device = login_device;
    [signing, login]
}

fn signing_and_login(device_a: &str, device_b: &str) -> [CertCandidate; 2] {
    on_devices(token(device_a), token(device_b))
}

#[test]
fn the_login_sibling_of_a_signing_certificate_on_the_same_device_is_hidden() {
    let candidates = signing_and_login("Card", "Card");
    let list = build_cert_list(&candidates, &context());
    assert_eq!(usable(&list), vec![fp(1)]);
    assert_eq!(
        hidden_reason(&list, fp(2)),
        Some(HiddenReason::LoginSibling)
    );
}

#[test]
fn login_certificates_on_another_device_are_kept() {
    let candidates = signing_and_login("Card A", "Card B");
    let list = build_cert_list(&candidates, &context());
    assert_eq!(list.usable.len(), 2);
    assert!(list.hidden.is_empty());
}

#[test]
fn login_certificates_of_another_holder_are_kept() {
    let [signing, mut login] = signing_and_login("Card", "Card");
    login = with_info(login, |i| {
        i.subject.common_name = Some("Maria Silva".to_owned())
    });
    let list = build_cert_list(&[signing, login], &context());
    assert_eq!(list.usable.len(), 2);
}

#[test]
fn a_login_certificate_alone_is_kept() {
    let [_, login] = signing_and_login("Card", "Card");
    let list = build_cert_list(&[login], &context());
    assert_eq!(usable(&list), vec![fp(2)]);
}

#[test]
fn two_signing_certificates_of_one_holder_are_both_kept() {
    let mut a = candidate(1, "Joao Pereira");
    a.device = token("Card");
    let mut b = candidate(2, "Joao Pereira");
    b.device = token("Card");
    let list = build_cert_list(&[a, b], &context());
    assert_eq!(list.usable.len(), 2);
}

#[test]
fn unsupported_key_wins_over_login_sibling() {
    let [signing, mut login] = signing_and_login("Card", "Card");
    login = with_info(login, |i| {
        i.key = PublicKeyKind::Unsupported {
            oid: "1.2.3".to_owned(),
        }
    });
    let list = build_cert_list(&[signing, login], &context());
    assert_eq!(
        hidden_reason(&list, fp(2)),
        Some(HiddenReason::UnsupportedKey)
    );
}

#[test]
fn two_certificates_without_a_known_device_count_as_the_same_place() {
    let list = build_cert_list(&on_devices(None, None), &context());
    assert_eq!(usable(&list), vec![fp(1)]);
    assert_eq!(
        hidden_reason(&list, fp(2)),
        Some(HiddenReason::LoginSibling)
    );
}

#[test]
fn a_known_device_and_an_unknown_one_are_different_places() {
    let list = build_cert_list(&on_devices(token("Card"), None), &context());
    assert_eq!(list.usable.len(), 2);
    assert!(list.hidden.is_empty());
}
