//! SPEC §10 (`docs/ux.md` §5.2, vectors §16.3): `present::holder`.

mod display_name;
mod title_case;

#[path = "../common/mod.rs"]
mod common;

use common::{blank_info, info};
use websign_core::CertInfo;
use websign_core::IcpBrasil;
use websign_core::present::holder::{display_name, title_case};

fn with_cn(cn: &str) -> CertInfo {
    let mut info = blank_info();
    info.subject.common_name = Some(cn.to_owned());
    info
}

/// An ICP-Brasil certificate the way the cert reader would summarize it.
fn icp_with_cn(cn: &str) -> CertInfo {
    let mut info = with_cn(cn);
    info.icp_brasil = Some(IcpBrasil {
        level: None,
        holder_name: Some(cn.rsplit_once(':').map_or(cn, |(name, _)| name).to_owned()),
        cpf: None,
        cnpj: None,
    });
    info
}
