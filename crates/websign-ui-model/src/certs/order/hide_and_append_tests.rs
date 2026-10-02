//! Hidden certificates and merging a later listing into an open list.

use websign_core::{KeyUsage, PublicKeyKind};

use super::*;
use crate::certs::{DeviceLabel, DisabledReason, HiddenReason, RowStatus};
use crate::fixtures::{NOW, candidate, context, fingerprint};

fn names(rows: &[CertRow]) -> Vec<&str> {
    rows.iter().map(|row| row.name.as_str()).collect()
}

fn info_mut(candidate: &mut CertCandidate) -> &mut websign_core::CertInfo {
    candidate.info.as_mut().expect("fixture parses")
}

#[test]
fn hides_by_certificate_content() {
    let mut ca = candidate(1, "CA");
    info_mut(&mut ca).is_ca = true;
    let mut usage = candidate(2, "Enc");
    info_mut(&mut usage).key_usage = Some(KeyUsage {
        key_encipherment: true,
        ..Default::default()
    });
    let mut tls = candidate(3, "Tls");
    info_mut(&mut tls).extended_key_usage = vec!["1.3.6.1.5.5.7.3.1".into()];
    let mut mixed = candidate(4, "Mixed");
    info_mut(&mut mixed).extended_key_usage =
        vec!["1.3.6.1.5.5.7.3.1".into(), "1.3.6.1.5.5.7.3.4".into()];
    let mut odd = candidate(5, "Odd");
    info_mut(&mut odd).key = PublicKeyKind::Unsupported {
        oid: "1.3.101.112".into(),
    };
    let list = build_cert_list(&[ca, usage, tls, mixed, odd], &context());
    assert_eq!(list.usable.len(), 1);
    let reasons: Vec<_> = list.hidden.iter().map(|(_, reason)| *reason).collect();
    assert_eq!(
        reasons,
        [
            HiddenReason::CertificateAuthority,
            HiddenReason::KeyUsage,
            HiddenReason::ExtendedKeyUsage,
            HiddenReason::UnsupportedKey,
        ]
    );
}

#[test]
fn login_sibling_needs_same_device() {
    let token = Some(DeviceLabel::Token { name: "T".into() });
    let mut login = candidate(1, "Rui");
    login.device = token.clone();
    info_mut(&mut login).key_usage = Some(KeyUsage {
        digital_signature: true,
        ..Default::default()
    });
    let mut signing = candidate(2, "Rui");
    signing.device = token;
    let mut elsewhere = candidate(3, "Rui");
    info_mut(&mut elsewhere).key_usage = login.info.as_ref().unwrap().key_usage;
    let list = build_cert_list(&[login, signing, elsewhere], &context());
    assert_eq!(list.hidden, [(fingerprint(1), HiddenReason::LoginSibling)]);
    assert_eq!(list.usable.len(), 2);
}

#[test]
fn append_keeps_order_and_selection_and_adds_at_the_end() {
    let mut list = build_cert_list(&[candidate(2, "Bia"), candidate(3, "Caio")], &context());
    list.selected = Some(fingerprint(3));
    let newcomer = candidate(1, "Ana");
    let mut expired = candidate(4, "Old");
    info_mut(&mut expired).not_after = NOW - 1;
    list.append(
        &[candidate(2, "Bia"), candidate(3, "Caio"), newcomer, expired],
        &context(),
    );
    assert_eq!(names(&list.usable), ["Bia", "Caio", "Ana"]);
    assert_eq!(list.selected, Some(fingerprint(3)));
    assert_eq!(list.disabled.len(), 1);
}

#[test]
fn removed_row_is_disabled_in_place_and_returns() {
    let mut list = build_cert_list(&[candidate(1, "Ana"), candidate(2, "Bia")], &context());
    let mut gone = candidate(1, "Ana");
    gone.removed = true;
    list.append(&[gone, candidate(2, "Bia")], &context());
    assert_eq!(list.usable[0].candidate.fingerprint, fingerprint(1));
    assert_eq!(
        list.usable[0].status,
        RowStatus::Disabled(DisabledReason::Removed)
    );
    assert_eq!(list.selected, Some(fingerprint(1)));
    list.append(&[candidate(1, "Ana"), candidate(2, "Bia")], &context());
    assert_eq!(list.usable[0].status, RowStatus::Usable);
    assert_eq!(list.usable.len(), 2);
}
