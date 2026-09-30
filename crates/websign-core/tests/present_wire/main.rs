//! SPEC §13: `present::wire::certificate_profile`, plus a round trip over the
//! total mappings that were already in place.

mod eidas;
mod icp_brasil;
mod profile;

#[path = "../common/mod.rs"]
mod common;

use common::{blank_info, info};
use websign_core::present::wire::{
    algorithm_name, certificate_profile, curve_name, hash_algorithm, hash_name, key_description,
    signature_algorithm,
};
use websign_core::{
    CertInfo, Curve, HashAlgorithm, IcpBrasil, IcpLevel, PublicKeyKind, QcType, Qualified,
    SignatureAlgorithm,
};
use websign_protocol::types::{CertificateProfile, EidasProfile, EidasType, KeyStorage};

fn icp_info(level: Option<IcpLevel>) -> CertInfo {
    let mut info = blank_info();
    info.icp_brasil = Some(IcpBrasil {
        level,
        holder_name: None,
        cpf: None,
        cnpj: None,
    });
    info
}

fn qualified_info(compliance: bool, sscd: bool, types: Vec<QcType>) -> CertInfo {
    let mut info = blank_info();
    info.qualified = Some(Qualified {
        compliance,
        sscd,
        types,
    });
    info
}

fn eidas(qualified: bool, qscd: bool, types: Vec<EidasType>) -> Option<EidasProfile> {
    Some(EidasProfile {
        qualified,
        qscd,
        types,
    })
}
