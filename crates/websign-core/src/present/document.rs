//! The holder's document number, masked (`docs/ux.md` §5.5, vectors §16.4).
//!
//! The full CPF never leaves this module: not to the UI, not to logs, not to
//! diagnostics.

use crate::cert::CertInfo;

/// Characters of an ETSI national identifier left visible.
const NATIONAL_VISIBLE: usize = 3;
/// Mask character of every label.
const DOT: char = '•';

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
    let icp = info.icp_brasil.as_ref();
    if let Some(cpf) = icp.and_then(|icp| icp.cpf.as_deref()).and_then(cpf_label) {
        return Some(cpf);
    }
    if let Some(formatted) = icp.and_then(|icp| icp.formatted_cnpj()) {
        return Some(DocumentLabel::Cnpj { formatted });
    }
    info.subject
        .serial_number
        .as_deref()
        .and_then(national_label)
}

fn cpf_label(cpf: &str) -> Option<DocumentLabel> {
    if cpf.len() != 11 || !cpf.bytes().all(|b| b.is_ascii_digit()) {
        return None;
    }
    let (a, b) = (&cpf[3..6], &cpf[6..9]);
    Some(DocumentLabel::Cpf {
        masked: format!("{DOT}{DOT}{DOT}.{a}.{b}-{DOT}{DOT}"),
        visible: format!("{a} {b}"),
    })
}

/// `^[A-Z]{3}[A-Z]{2}-` (country and type, ETSI EN 319 412-1 §5.1.3) followed
/// by at least three characters.
fn national_label(serial: &str) -> Option<DocumentLabel> {
    let (prefix, number) = serial.split_once('-')?;
    if prefix.len() != 5 || !prefix.bytes().all(|b| b.is_ascii_uppercase()) {
        return None;
    }
    let count = number.chars().count();
    if count < NATIONAL_VISIBLE {
        return None;
    }
    let visible: String = number.chars().skip(count - NATIONAL_VISIBLE).collect();
    Some(DocumentLabel::National {
        masked: format!("{}{visible}", DOT.to_string().repeat(5)),
    })
}
