//! ICP-Brasil markers: certificate level and holder identifiers (DOC-ICP-04).

use std::fmt;

/// ICP-Brasil data extracted from policies and SubjectAltName `otherName`s.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct IcpBrasil {
    /// From the first `2.16.76.1.2.<n>` policy.
    pub level: Option<IcpLevel>,
    /// Subject CN without the trailing `:<digits>` ICP-Brasil appends.
    pub holder_name: Option<String>,
    /// 11 digits: the holder's CPF, or the CPF of a company's responsible person.
    pub cpf: Option<String>,
    /// 14 digits.
    pub cnpj: Option<String>,
}

impl IcpBrasil {
    /// CPF masked like gov.br does: `***.456.789-**`.
    ///
    /// Enough for the holder to recognize their own certificate, useless to
    /// anyone reading over their shoulder.
    pub fn masked_cpf(&self) -> Option<String> {
        todo!()
    }

    /// CNPJ in its usual `12.345.678/0001-95` form. A company identifier is
    /// public data, so it is not masked.
    pub fn formatted_cnpj(&self) -> Option<String> {
        todo!()
    }
}

/// Certificate type from the ICP-Brasil policy OID.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IcpLevel {
    A1,
    A2,
    A3,
    A4,
    S1,
    S2,
    S3,
    S4,
    T3,
    T4,
    /// A policy arc this crate does not know yet.
    Other(u32),
}

impl fmt::Display for IcpLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}
