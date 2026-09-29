//! Unit tests for `CertInfo`, grouped by what they exercise. Certificates come from
//! `testkit` (hand-built DER) except for one real OpenSSL-issued sample.

mod basics;
mod extensions;
mod keys;
mod names;
mod profiles;

use super::fixtures::ICP_LIKE_CERT;
use super::testkit::*;
use super::*;

pub(super) const JAN_1_2024: i64 = 1_704_067_200;
pub(super) const JAN_1_2025: i64 = 1_735_689_600;

pub(super) fn parse(cert: &TestCert) -> CertInfo {
    CertInfo::from_der(&cert.build()).expect("test certificate must parse")
}

pub(super) fn subject_of(rdns: &[Vec<u8>]) -> DistinguishedName {
    parse(&TestCert::new().subject(rdns)).subject
}

pub(super) fn icp_cert() -> TestCert {
    let person = tlv(0x0c, b"010119901234567890112345678901234");
    let company = tlv(0x13, b"12345678000195");
    TestCert::new()
        .subject(&[
            rdn(COUNTRY, PRINTABLE, b"BR"),
            rdn(ORG, UTF8, b"ICP-Brasil"),
            rdn(CN, UTF8, b"ANA BEATRIZ SOUZA:12345678901"),
        ])
        .extension(policies(&["2.16.76.1.2.3.15"]))
        .extension(san_other_names(&[
            ("2.16.76.1.3.1", person),
            ("2.16.76.1.3.3", company),
        ]))
        .extension(key_usage(&[0, 1, 2]))
}
