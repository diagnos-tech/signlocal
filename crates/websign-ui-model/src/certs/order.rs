//! Building the list: hide, disable, order, select (`docs/ux.md` §5.8,
//! §5.9, vectors §16.6).

use jiff::civil::Date;
use websign_core::{Fingerprint, SignatureAlgorithm};

use super::candidate::CertCandidate;
use super::row::CertList;

/// What the list depends on besides the candidates.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ListContext {
    /// Local date for validity labels.
    pub today: Date,
    /// Unix seconds, for validity checks.
    pub now: i64,
    /// Algorithms the request accepts, preferred first; empty = any.
    pub accepted: Vec<SignatureAlgorithm>,
    /// For a remembered caller: the certificate it used last.
    pub last_used_here: Option<Fingerprint>,
    /// Every certificate by most recent use anywhere (newest first).
    pub recent_anywhere: Vec<Fingerprint>,
    /// A certificate the request asked to preselect (`sign.begin.certificate`).
    pub requested: Option<Fingerprint>,
}

/// Builds the list from scratch. While a window is open, new certificates are
/// appended with [`CertList::append`] instead, so nothing reorders under the
/// pointer.
pub fn build_cert_list(candidates: &[CertCandidate], context: &ListContext) -> CertList {
    let _ = (candidates, context);
    todo!("SPEC.md §1.5")
}

impl CertList {
    /// Merges a fresh listing into an open window's list: rows keep their
    /// place; new usable rows go to the end of the usable group; rows whose
    /// token left become `Disabled(Removed)` in place; returning tokens
    /// re-enable their rows. The selection never moves by itself.
    pub fn append(&mut self, candidates: &[CertCandidate], context: &ListContext) {
        let _ = (candidates, context);
        todo!("SPEC.md §1.6")
    }
}
