//! The single neutral badge of a row (`docs/ux.md` §5.3).

use websign_core::CertInfo;

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
    let _ = info;
    todo!("SPEC.md §1.2")
}
