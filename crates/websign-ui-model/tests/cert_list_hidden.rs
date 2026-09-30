//! SPEC §1.5: what is never listed, and why (first reason wins).

mod common;

use common::*;
use websign_core::{CertError, KeyUsage, PublicKeyKind};
use websign_ui_model::certs::{CertCandidate, HiddenReason, build_cert_list};

/// Lists `candidate` alone and returns its hidden reason, if any.
fn hidden_alone(candidate: CertCandidate) -> Option<HiddenReason> {
    let fingerprint = candidate.fingerprint;
    let list = build_cert_list(&[candidate], &context());
    let listed = list.usable.len() + list.disabled.len();
    let hidden = hidden_reason(&list, fingerprint);
    assert_eq!(listed == 0, hidden.is_some(), "listed xor hidden");
    hidden
}

fn usage(digital_signature: bool, non_repudiation: bool, key_encipherment: bool) -> KeyUsage {
    KeyUsage {
        digital_signature,
        non_repudiation,
        key_encipherment,
        ..KeyUsage::default()
    }
}

fn unparseable() -> CertCandidate {
    let mut c = candidate(1, "x");
    c.info = Err(CertError::Malformed("bad".to_owned()));
    c
}

#[test]
fn a_plain_signing_certificate_is_listed() {
    assert_eq!(hidden_alone(candidate(1, "Ana")), None);
}

#[test]
fn unparseable_certificates_are_hidden() {
    assert_eq!(hidden_alone(unparseable()), Some(HiddenReason::Unparseable));
}

#[test]
fn certificates_without_a_private_key_are_hidden() {
    let mut c = candidate(1, "Ana");
    c.has_private_key = false;
    assert_eq!(hidden_alone(c), Some(HiddenReason::NoPrivateKey));
}

#[test]
fn certificate_authorities_are_hidden() {
    let c = with_info(candidate(1, "AC Raiz"), |i| i.is_ca = true);
    assert_eq!(hidden_alone(c), Some(HiddenReason::CertificateAuthority));
}

#[test]
fn key_usage_needs_digital_signature_or_non_repudiation() {
    let only_encipherment = with_info(candidate(1, "A"), |i| {
        i.key_usage = Some(usage(false, false, true))
    });
    assert_eq!(
        hidden_alone(only_encipherment),
        Some(HiddenReason::KeyUsage)
    );

    let only_signature = with_info(candidate(2, "B"), |i| {
        i.key_usage = Some(usage(true, false, false))
    });
    assert_eq!(hidden_alone(only_signature), None);

    let only_commitment = with_info(candidate(3, "C"), |i| {
        i.key_usage = Some(usage(false, true, false))
    });
    assert_eq!(hidden_alone(only_commitment), None);

    let no_extension = with_info(candidate(4, "D"), |i| i.key_usage = None);
    assert_eq!(hidden_alone(no_extension), None);
}

#[test]
fn certificates_for_foreign_purposes_only_are_hidden() {
    let foreign: [&[&str]; 3] = [
        &[SERVER_AUTH],
        &[CODE_SIGNING, TIME_STAMPING],
        &[SERVER_AUTH, CODE_SIGNING, TIME_STAMPING, OCSP_SIGNING],
    ];
    for oids in foreign {
        let c = with_info(candidate(1, "Servidor"), |i| {
            i.extended_key_usage = oids.iter().map(|s| (*s).to_owned()).collect();
        });
        assert_eq!(
            hidden_alone(c),
            Some(HiddenReason::ExtendedKeyUsage),
            "{oids:?}"
        );
    }
}

#[test]
fn one_acceptable_purpose_keeps_the_certificate() {
    let passing: [&[&str]; 7] = [
        &[],
        &[ANY_EKU],
        &[CLIENT_AUTH],
        &[EMAIL_PROTECTION],
        &[DOCUMENT_SIGNING],
        &[MS_DOCUMENT_SIGNING],
        &[SERVER_AUTH, CLIENT_AUTH],
    ];
    for oids in passing {
        let c = with_info(candidate(1, "Ana"), |i| {
            i.extended_key_usage = oids.iter().map(|s| (*s).to_owned()).collect();
        });
        assert_eq!(hidden_alone(c), None, "{oids:?}");
    }
}

#[test]
fn an_unknown_purpose_is_not_foreign() {
    // Hidden only when *every* OID is in the foreign set.
    let c = with_info(candidate(1, "Ana"), |i| {
        i.extended_key_usage = vec!["1.2.3.4.5".to_owned(), SERVER_AUTH.to_owned()];
    });
    assert_eq!(hidden_alone(c), None);
}

#[test]
fn keys_the_app_cannot_sign_with_are_hidden() {
    let c = with_info(candidate(1, "Ana"), |i| {
        i.key = PublicKeyKind::Unsupported {
            oid: "1.3.101.112".to_owned(),
        };
    });
    assert_eq!(hidden_alone(c), Some(HiddenReason::UnsupportedKey));
}

#[test]
fn ec_keys_are_listed() {
    let c = with_info(candidate(1, "Ana"), |i| {
        i.key = PublicKeyKind::Ec {
            curve: websign_core::Curve::P256,
        };
    });
    assert_eq!(hidden_alone(c), None);
}

#[test]
fn the_first_matching_reason_wins() {
    // No private key beats CA.
    let mut c = with_info(candidate(1, "A"), |i| i.is_ca = true);
    c.has_private_key = false;
    assert_eq!(hidden_alone(c), Some(HiddenReason::NoPrivateKey));

    // CA beats key usage.
    let c = with_info(candidate(2, "B"), |i| {
        i.is_ca = true;
        i.key_usage = Some(usage(false, false, true));
    });
    assert_eq!(hidden_alone(c), Some(HiddenReason::CertificateAuthority));

    // Key usage beats extended key usage.
    let c = with_info(candidate(3, "C"), |i| {
        i.key_usage = Some(usage(false, false, true));
        i.extended_key_usage = vec![SERVER_AUTH.to_owned()];
    });
    assert_eq!(hidden_alone(c), Some(HiddenReason::KeyUsage));

    // Extended key usage beats the unsupported key.
    let c = with_info(candidate(4, "D"), |i| {
        i.extended_key_usage = vec![SERVER_AUTH.to_owned()];
        i.key = PublicKeyKind::Unsupported {
            oid: "1.2.3".to_owned(),
        };
    });
    assert_eq!(hidden_alone(c), Some(HiddenReason::ExtendedKeyUsage));
}

#[test]
fn a_hidden_certificate_is_never_shown_as_disabled_even_when_expired() {
    let mut c = with_info(candidate(1, "Ana"), |i| i.not_after = noon(2020, 1, 1));
    c.has_private_key = false;
    assert_eq!(hidden_alone(c), Some(HiddenReason::NoPrivateKey));
}
