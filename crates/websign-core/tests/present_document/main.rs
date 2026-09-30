//! SPEC §11 (`docs/ux.md` §5.5, vectors §16.4): `present::document`.

mod certificates;
mod etsi;
mod privacy;
mod rules;

#[path = "../common/mod.rs"]
mod common;

use common::{blank_info, cert, cert_names, info};
use websign_core::present::document::{DocumentLabel, display_document};
use websign_core::{CertInfo, IcpBrasil};

fn icp(cpf: Option<&str>, cnpj: Option<&str>) -> CertInfo {
    let mut info = blank_info();
    info.icp_brasil = Some(IcpBrasil {
        level: None,
        holder_name: None,
        cpf: cpf.map(str::to_owned),
        cnpj: cnpj.map(str::to_owned),
    });
    info
}

fn with_serial(serial: &str) -> CertInfo {
    let mut info = blank_info();
    info.subject.serial_number = Some(serial.to_owned());
    info
}

fn cpf_label(masked: &str, visible: &str) -> DocumentLabel {
    DocumentLabel::Cpf {
        masked: masked.to_owned(),
        visible: visible.to_owned(),
    }
}

fn national(masked: &str) -> DocumentLabel {
    DocumentLabel::National {
        masked: masked.to_owned(),
    }
}

fn cnpj_label(formatted: &str) -> DocumentLabel {
    DocumentLabel::Cnpj {
        formatted: formatted.to_owned(),
    }
}
