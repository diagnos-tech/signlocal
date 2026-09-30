use websign_core::{KeyUsage, PublicKeyKind, SignatureAlgorithm};

use super::*;
use crate::certs::{DeviceLabel, DisabledReason, HiddenReason, KeySource, PinMode, RowStatus};
use crate::fixtures::{NOW, candidate, context, fingerprint};

fn names(rows: &[CertRow]) -> Vec<&str> {
    rows.iter().map(|row| row.name.as_str()).collect()
}

fn info_mut(candidate: &mut CertCandidate) -> &mut websign_core::CertInfo {
    candidate.info.as_mut().expect("fixture parses")
}

#[test]
fn spec_vector_16_6() {
    let mut a3 = candidate(1, "Ana Souza");
    a3.alternates = vec![KeySource::Driver {
        path: "~/opensc.so".into(),
    }];
    let mut a1_ana = candidate(2, "Ana Souza");
    a1_ana.hardware = Some(false);
    let mut a1_clinic = candidate(3, "Clinica Aurora");
    a1_clinic.hardware = Some(false);
    let mut expired = candidate(4, "Ana Souza");
    info_mut(&mut expired).not_after = NOW - 86_400 * 30;
    let mut login = candidate(5, "Rui Costa");
    info_mut(&mut login).key_usage = Some(KeyUsage {
        digital_signature: true,
        ..Default::default()
    });
    let mut signing = candidate(6, "Rui Costa");
    signing.hardware = Some(false);
    let mut no_key = candidate(7, "Sem Chave");
    no_key.has_private_key = false;

    let list = build_cert_list(
        &[a3, a1_ana, a1_clinic, expired, login, signing, no_key],
        &context(),
    );

    let usable: Vec<_> = list
        .usable
        .iter()
        .map(|row| row.candidate.fingerprint)
        .collect();
    assert_eq!(
        usable[..3],
        [fingerprint(1), fingerprint(2), fingerprint(3)]
    );
    assert_eq!(list.usable[0].candidate.alternates.len(), 1);
    assert_eq!(
        list.disabled[0].status,
        RowStatus::Disabled(DisabledReason::Expired)
    );
    assert!(
        list.hidden
            .contains(&(fingerprint(5), HiddenReason::LoginSibling))
    );
    assert!(
        list.hidden
            .contains(&(fingerprint(7), HiddenReason::NoPrivateKey))
    );
    assert_eq!(list.selected, Some(fingerprint(1)));
}

#[test]
fn last_used_here_comes_first_and_is_selected() {
    let a3 = candidate(1, "Ana Souza");
    let mut a1 = candidate(2, "Ana Souza");
    a1.hardware = Some(false);
    let mut ctx = context();
    ctx.last_used_here = Some(fingerprint(2));
    let list = build_cert_list(&[a3, a1], &ctx);
    assert_eq!(list.usable[0].candidate.fingerprint, fingerprint(2));
    assert_eq!(list.selected, Some(fingerprint(2)));
}

#[test]
fn recent_anywhere_precedes_never_used() {
    let list = build_cert_list(
        &[
            candidate(1, "Ana"),
            candidate(2, "Bia"),
            candidate(3, "Caio"),
        ],
        &ListContext {
            recent_anywhere: vec![fingerprint(3), fingerprint(2)],
            ..context()
        },
    );
    assert_eq!(names(&list.usable), ["Caio", "Bia", "Ana"]);
}

#[test]
fn never_used_orders_hardware_then_name_then_validity() {
    let mut software = candidate(1, "Aaa");
    software.hardware = Some(false);
    let unknown = {
        let mut c = candidate(2, "Bbb");
        c.hardware = None;
        c
    };
    let late = candidate(3, "Ccc");
    let mut early = candidate(4, "Ccc");
    info_mut(&mut early).not_after -= 1000;
    let list = build_cert_list(&[software, unknown, early, late], &context());
    let order: Vec<_> = list
        .usable
        .iter()
        .map(|row| row.candidate.fingerprint)
        .collect();
    assert_eq!(
        order,
        [
            fingerprint(3),
            fingerprint(4),
            fingerprint(2),
            fingerprint(1)
        ]
    );
}

#[test]
fn requested_beats_last_used_when_usable() {
    let mut expired = candidate(3, "Old");
    info_mut(&mut expired).not_after = NOW - 10;
    let candidates = [candidate(1, "Ana"), candidate(2, "Bia"), expired];
    let mut ctx = context();
    ctx.last_used_here = Some(fingerprint(1));
    ctx.requested = Some(fingerprint(2));
    assert_eq!(
        build_cert_list(&candidates, &ctx).selected,
        Some(fingerprint(2))
    );
    ctx.requested = Some(fingerprint(3));
    assert_eq!(
        build_cert_list(&candidates, &ctx).selected,
        Some(fingerprint(1))
    );
    ctx.last_used_here = None;
    ctx.requested = None;
    assert_eq!(
        build_cert_list(&candidates, &ctx).selected,
        Some(fingerprint(1))
    );
    assert_eq!(build_cert_list(&[], &ctx).selected, None);
}

#[test]
fn disabled_reasons_in_priority_order() {
    let mut removed = candidate(1, "A");
    removed.removed = true;
    info_mut(&mut removed).not_after = NOW - 1;
    let mut not_yet = candidate(2, "B");
    info_mut(&mut not_yet).not_before = NOW + 100;
    let mut locked = candidate(3, "C");
    locked.pin = PinMode::App {
        length: None,
        count_low: false,
        final_try: false,
        locked: true,
    };
    let incompatible = candidate(4, "D");
    let ctx = ListContext {
        accepted: vec![SignatureAlgorithm::Ecdsa],
        ..context()
    };
    let list = build_cert_list(&[removed, not_yet, locked, incompatible], &ctx);
    let reasons: Vec<_> = list.disabled.iter().map(|row| row.status).collect();
    assert_eq!(
        reasons,
        [
            RowStatus::Disabled(DisabledReason::Removed),
            RowStatus::Disabled(DisabledReason::NotYetValid),
            RowStatus::Disabled(DisabledReason::PinLocked),
            RowStatus::Disabled(DisabledReason::Incompatible),
        ]
    );
    assert!(list.usable.is_empty());
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
