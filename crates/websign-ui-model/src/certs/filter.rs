//! The filter field shown above long lists (`docs/ux.md` §5.13).

use super::row::CertRow;

/// The filter field appears with more usable rows than this.
pub const FILTER_THRESHOLD: usize = 6;

/// Whether `row` matches `query`: case- and accent-insensitive substring of
/// the name or issuer, or digits of the *visible* part of the document.
pub fn matches_filter(row: &CertRow, query: &str) -> bool {
    let _ = (row, query);
    todo!("SPEC.md §1.7")
}
