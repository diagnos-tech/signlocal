//! The single neutral badge of a row (`docs/ux.md` §5.3).

use websign_core::{CertInfo, IcpLevel};

use super::text::fold;

/// First matching rule of the §5.3 table.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Badge {
    /// Portuguese Cartão de Cidadão (issuer CN contains "Cartão de Cidadão").
    PtCitizenCard,
    /// Spanish DNIe (issuer O = "DIRECCION GENERAL DE LA POLICIA").
    EsDnie,
    /// ICP-Brasil with a class: `"A3"`, `"S1"`, `"T3"`…
    IcpBrasil {
        class: String,
    },
    /// ICP-Brasil without a recognized class.
    IcpBrasilPlain,
    /// QcCompliance and QcSSCD.
    EidasQualified,
    /// QcCompliance without QcSSCD.
    Eidas,
    Generic,
}

/// The badge for `info`.
pub fn badge(info: &CertInfo) -> Badge {
    let issuer_cn = info.issuer.common_name.as_deref().map(fold);
    if issuer_cn.is_some_and(|cn| cn.contains("cartao de cidadao")) {
        return Badge::PtCitizenCard;
    }
    let issuer_org = info.issuer.organization.as_deref().map(fold);
    if issuer_org.is_some_and(|org| org == "direccion general de la policia") {
        return Badge::EsDnie;
    }
    if let Some(icp) = &info.icp_brasil {
        return match icp.level.and_then(class_name) {
            Some(class) => Badge::IcpBrasil {
                class: class.to_owned(),
            },
            None => Badge::IcpBrasilPlain,
        };
    }
    match &info.qualified {
        Some(qc) if qc.compliance && qc.sscd => Badge::EidasQualified,
        Some(qc) if qc.compliance => Badge::Eidas,
        _ => Badge::Generic,
    }
}

/// The class shown in the badge; levels outside the table have none.
fn class_name(level: IcpLevel) -> Option<&'static str> {
    Some(match level {
        IcpLevel::A1 => "A1",
        IcpLevel::A2 => "A2",
        IcpLevel::A3 => "A3",
        IcpLevel::A4 => "A4",
        IcpLevel::S1 => "S1",
        IcpLevel::S2 => "S2",
        IcpLevel::S3 => "S3",
        IcpLevel::S4 => "S4",
        IcpLevel::T3 => "T3",
        IcpLevel::T4 => "T4",
        IcpLevel::Other(_) => return None,
    })
}
