//! Unit tests for the text rules, over the vectors of `docs/ux.md` §16.

mod caller;
mod document;
mod holder;
mod origin;
mod wire;

use crate::CertInfo;
use crate::testkit::*;

/// An ICP-Brasil person certificate (A3) with the given CN, CPF and CNPJ.
pub(super) fn icp_info(cn: &str, cpf: Option<&str>, cnpj: Option<&str>) -> CertInfo {
    let mut names = Vec::new();
    if let Some(cpf) = cpf {
        let value = format!("01011990{cpf}00000000000000000000");
        names.push(("2.16.76.1.3.1", tlv(UTF8, value.as_bytes())));
    }
    if let Some(cnpj) = cnpj {
        names.push(("2.16.76.1.3.3", tlv(PRINTABLE, cnpj.as_bytes())));
    }
    let cert = TestCert::new()
        .subject(&[rdn(CN, UTF8, cn.as_bytes())])
        .extension(policies(&["2.16.76.1.2.3.15"]))
        .extension(san_other_names(&names));
    CertInfo::from_der(&cert.build()).expect("test certificate parses")
}

pub(super) fn info_with(rdns: &[Vec<u8>]) -> CertInfo {
    CertInfo::from_der(&TestCert::new().subject(rdns).build()).expect("test certificate parses")
}
