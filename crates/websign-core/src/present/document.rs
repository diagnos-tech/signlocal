//! The holder's document number, masked (`docs/ux.md` §5.5, vectors §16.4).
//!
//! The full CPF never leaves this module: not to the UI, not to logs, not to
//! diagnostics.

use crate::cert::CertInfo;

/// What the second line of a certificate row shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentLabel {
    /// ICP-Brasil person: `masked` = `"•••.456.789-••"` (digits 4 to 9, the
    /// gov.br mask); `visible` = `"456 789"` for screen readers.
    Cpf { masked: String, visible: String },
    /// ICP-Brasil company: `"12.345.678/0001-90"`, in full (public registry data).
    Cnpj { formatted: String },
    /// ETSI national identifier from the subject `serialNumber`
    /// (`IDCPT-12345123` → `"•••••123"`).
    National { masked: String },
}

/// The document to show for `info`, in the order CPF, CNPJ, ETSI serial
/// number; `None` when there is none.
pub fn display_document(info: &CertInfo) -> Option<DocumentLabel> {
    let _ = info;
    todo!("SPEC.md §11")
}
