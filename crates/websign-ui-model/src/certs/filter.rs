//! The filter field shown above long lists (`docs/ux.md` §5.13).

use websign_core::present::document::DocumentLabel;

use super::row::CertRow;
use super::text::fold;

/// The filter field appears with more usable rows than this.
pub const FILTER_THRESHOLD: usize = 6;

/// Whether `row` matches `query`: case- and accent-insensitive substring of
/// the name or issuer, or digits of the *visible* part of the document.
pub fn matches_filter(row: &CertRow, query: &str) -> bool {
    let query = query.trim();
    if query.is_empty() {
        return true;
    }
    let needle = fold(query);
    if fold(&row.name).contains(&needle) || fold(&row.issuer).contains(&needle) {
        return true;
    }
    let digits = query_digits(query);
    !digits.is_empty() && document_digits(row).contains(&digits)
}

/// The query as digits, when it is made only of digits and the separators
/// people type in a CPF or CNPJ; empty otherwise.
fn query_digits(query: &str) -> String {
    let is_separator = |c: char| matches!(c, '.' | '-' | '/' | ' ');
    let stripped: String = query.chars().filter(|&c| !is_separator(c)).collect();
    if stripped.chars().all(|c| c.is_ascii_digit()) {
        stripped
    } else {
        String::new()
    }
}

/// Digits of the part of the document the row shows.
fn document_digits(row: &CertRow) -> String {
    let shown = match &row.document {
        Some(DocumentLabel::Cpf { visible, .. }) => visible,
        Some(DocumentLabel::Cnpj { formatted }) => formatted,
        Some(DocumentLabel::National { masked }) => masked,
        None => return String::new(),
    };
    shown.chars().filter(char::is_ascii_digit).collect()
}
