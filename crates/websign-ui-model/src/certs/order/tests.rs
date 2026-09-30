//! Ordering, selection and disabled reasons of a fresh list.

use websign_core::{KeyUsage, SignatureAlgorithm};

use super::*;
use crate::certs::{DisabledReason, HiddenReason, KeySource, PinMode, RowStatus};
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
