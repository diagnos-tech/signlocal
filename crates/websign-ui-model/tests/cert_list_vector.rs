//! ux §16.6 / SPEC §1.5 vector: Ana's certificates, the clinic's, an old one,
//! a Cartão de Cidadão pair and a certificate without a key.

mod common;

use common::*;
use websign_core::KeyUsage;
use websign_ui_model::certs::{
    CertCandidate, DisabledReason, HiddenReason, ListContext, RowStatus, build_cert_list,
};

const A3_ANA: u8 = 1;
const A1_ANA: u8 = 2;
const A1_CLINIC: u8 = 3;
const OLD_A3: u8 = 4;
const CC_LOGIN: u8 = 5;
const CC_SIGN: u8 = 6;
const NO_KEY: u8 = 7;

fn vector() -> Vec<CertCandidate> {
    // Ana's A3 seen through Windows and through the driver is ONE candidate
    // with an alternate (deduplicated by fingerprint before it gets here).
    let mut a3 = candidate(A3_ANA, "Ana Beatriz Souza");
    a3.hardware = Some(true);
    a3.device = token("SafeNet eToken 5110");
    a3.alternates = vec![driver("eTPKCS11.dll")];

    let a1_ana = candidate(A1_ANA, "Ana Beatriz Souza");
    let a1_clinic = candidate(A1_CLINIC, "Clinica Diagnos Ltda");

    let mut old = with_info(candidate(OLD_A3, "Ana Beatriz Souza"), |i| {
        i.not_after = noon(2026, 5, 10);
    });
    old.hardware = Some(true);

    let card = token("Cartao de Cidadao");
    let mut login = with_info(candidate(CC_LOGIN, "Joao Pereira"), |i| {
        i.key_usage = Some(KeyUsage {
            digital_signature: true,
            ..KeyUsage::default()
        });
    });
    login.device = card.clone();
    login.hardware = Some(true);
    let mut sign = candidate(CC_SIGN, "Joao Pereira");
    sign.device = card;
    sign.hardware = Some(true);

    let mut no_key = candidate(NO_KEY, "Sem Chave");
    no_key.has_private_key = false;

    vec![a1_clinic, no_key, login, old, a1_ana, sign, a3]
}

fn context_with_last_used(last_used_here: Option<u8>) -> ListContext {
    let mut c = context();
    c.last_used_here = last_used_here.map(fp);
    c
}

#[test]
fn groups_and_reasons_match_the_vector() {
    let list = build_cert_list(&vector(), &context());
    let mut usable_set = usable(&list);
    usable_set.sort();
    assert_eq!(
        usable_set,
        vec![fp(A3_ANA), fp(A1_ANA), fp(A1_CLINIC), fp(CC_SIGN)]
    );
    assert_eq!(disabled(&list), vec![fp(OLD_A3)]);
    assert_eq!(
        status(&list, fp(OLD_A3)),
        RowStatus::Disabled(DisabledReason::Expired)
    );
    assert_eq!(
        hidden_reason(&list, fp(CC_LOGIN)),
        Some(HiddenReason::LoginSibling)
    );
    assert_eq!(
        hidden_reason(&list, fp(NO_KEY)),
        Some(HiddenReason::NoPrivateKey)
    );
    assert_eq!(list.hidden.len(), 2);
}

#[test]
fn the_deduplicated_a3_keeps_its_alternate_path() {
    let list = build_cert_list(&vector(), &context());
    let a3 = row(&list, fp(A3_ANA));
    assert_eq!(a3.candidate.alternates, vec![driver("eTPKCS11.dll")]);
}

#[test]
fn never_used_order_is_hardware_then_name_then_validity() {
    let list = build_cert_list(&vector(), &context());
    // Hardware: A3 (Ana), CC signing (Joao); software: A1 Ana, A1 clinic.
    assert_eq!(
        usable(&list),
        vec![fp(A3_ANA), fp(CC_SIGN), fp(A1_ANA), fp(A1_CLINIC)]
    );
    assert_eq!(list.selected, Some(fp(A3_ANA)));
}

#[test]
fn last_used_here_comes_first_and_is_selected() {
    let list = build_cert_list(&vector(), &context_with_last_used(Some(A1_ANA)));
    assert_eq!(
        usable(&list),
        vec![fp(A1_ANA), fp(A3_ANA), fp(CC_SIGN), fp(A1_CLINIC)]
    );
    assert_eq!(list.selected, Some(fp(A1_ANA)));
}

#[test]
fn candidate_input_order_does_not_change_the_result() {
    let forward = build_cert_list(&vector(), &context());
    let mut reversed = vector();
    reversed.reverse();
    let backward = build_cert_list(&reversed, &context());
    assert_eq!(usable(&forward), usable(&backward));
    assert_eq!(disabled(&forward), disabled(&backward));
    assert_eq!(forward.selected, backward.selected);
}
