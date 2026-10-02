//! SPEC §6.1a: `given_name`, `surname` and `serial_number` of the subject.
//!
//! The names and numbers are invented (`tests/fixtures/gen/certs-personal.sh`).

mod common;

use common::{blank_info, cert_names, info};
use websign_core::CertInfo;

fn subject_of(name: &str) -> (Option<String>, Option<String>, Option<String>) {
    let subject = info(name).subject;
    (subject.given_name, subject.surname, subject.serial_number)
}

fn some(text: &str) -> Option<String> {
    Some(text.to_owned())
}

#[test]
fn reads_given_name_surname_and_serial_number() {
    assert_eq!(
        subject_of("dn-pii-only"),
        (some("MARIA"), some("SILVA"), some("IDCPT-12345123"))
    );
}

#[test]
fn reads_the_new_attributes_next_to_the_old_ones() {
    let subject = info("dn-pii").subject;
    assert_eq!(
        subject.common_name.as_deref(),
        Some("JOSÉ ÂNGELO CONCEIÇÃO")
    );
    assert_eq!(
        subject.organization.as_deref(),
        Some("WebeSign Test Fixtures")
    );
    assert_eq!(subject.country.as_deref(), Some("PT"));
    assert_eq!(subject.given_name.as_deref(), Some("José Ângelo"));
    assert_eq!(subject.surname.as_deref(), Some("Conceição"));
    assert_eq!(subject.serial_number.as_deref(), Some("IDCPT-12345123"));
}

#[test]
fn a_subject_without_them_has_none() {
    for name in [
        "rsa2048",
        "dn-empty",
        "dn-utf8",
        "icp-pf-a3",
        "qc-esign-sscd",
        "ca",
    ] {
        assert_eq!(subject_of(name), (None, None, None), "{name}");
    }
}

#[test]
fn the_new_attributes_are_read_from_the_subject_only() {
    let issuer = info("dn-pii").issuer;
    assert_eq!(
        (issuer.given_name, issuer.surname, issuer.serial_number),
        (None, None, None)
    );
}

#[test]
fn the_first_value_of_each_attribute_wins() {
    assert_eq!(
        subject_of("dn-pii-duplicates"),
        (some("First"), some("One"), some("IDCPT-1111111"))
    );
}

#[test]
fn the_first_readable_value_wins_over_an_ignored_one() {
    // The first givenName, surname and serialNumber are NumericStrings.
    assert_eq!(
        subject_of("dn-pii-unreadable-first"),
        (some("Second"), some("Beta"), some("IDCPT-2222222"))
    );
}

#[test]
fn decodes_bmp_strings_like_the_other_attributes() {
    let (given, surname, serial) = subject_of("dn-pii-bmp");
    assert_eq!(given.as_deref(), Some("JOSÉ Ω"));
    assert_eq!(surname.as_deref(), Some("ÊXITO"));
    assert_eq!(serial, None);
}

#[test]
fn display_name_uses_given_name_and_surname_after_the_common_name() {
    // SPEC §6.5: holder, CN, givenName + surname, O, fingerprint prefix.
    assert_eq!(
        info("dn-pii").display_name(),
        "JOSÉ ÂNGELO CONCEIÇÃO",
        "CN wins"
    );
    assert_eq!(info("dn-pii-only").display_name(), "MARIA SILVA");
    assert_eq!(info("dn-pii-bmp").display_name(), "Bmp Person");
}

#[test]
fn every_new_fixture_is_a_readable_certificate() {
    for name in cert_names() {
        if name.starts_with("dn-pii") {
            assert!(CertInfo::from_der(&common::cert(&name)).is_ok(), "{name}");
        }
    }
    assert_eq!(blank_info().subject.given_name, None);
}
