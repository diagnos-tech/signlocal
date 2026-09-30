//! Building the list: hide, disable, order, select (`docs/ux.md` §5.8,
//! §5.9, vectors §16.6).

use std::cmp::Reverse;

use jiff::civil::Date;
use websign_core::{Fingerprint, SignatureAlgorithm};

use super::build_row::make_row;
use super::candidate::CertCandidate;
use super::hide::hidden_reason;
use super::row::{CertList, CertRow, HiddenReason, RowStatus};
use super::status::row_status;
use super::text::fold;

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

/// Rows and hidden entries of the candidates a list has not seen yet.
pub(super) struct Sorted {
    pub usable: Vec<CertRow>,
    pub disabled: Vec<CertRow>,
    pub hidden: Vec<(Fingerprint, HiddenReason)>,
}

/// Builds the list from scratch. While a window is open, new certificates are
/// appended with [`CertList::append`] instead, so nothing reorders under the
/// pointer.
pub fn build_cert_list(candidates: &[CertCandidate], context: &ListContext) -> CertList {
    let Sorted {
        usable,
        disabled,
        hidden,
    } = classify(candidates, context, |_| false);
    let is_usable = |fingerprint: &Fingerprint| {
        usable
            .iter()
            .any(|row| row.candidate.fingerprint == *fingerprint)
    };
    let selected = [context.requested, context.last_used_here]
        .into_iter()
        .flatten()
        .find(is_usable)
        .or_else(|| usable.first().map(|row| row.candidate.fingerprint));
    CertList {
        usable,
        disabled,
        hidden,
        selected,
    }
}

/// Splits `candidates` (skipping those `known` already) into ordered usable
/// rows, ordered disabled rows and hidden entries.
pub(super) fn classify(
    candidates: &[CertCandidate],
    context: &ListContext,
    known: impl Fn(&Fingerprint) -> bool,
) -> Sorted {
    let mut sorted = Sorted {
        usable: Vec::new(),
        disabled: Vec::new(),
        hidden: Vec::new(),
    };
    for candidate in candidates.iter().filter(|c| !known(&c.fingerprint)) {
        if let Some(reason) = hidden_reason(candidate, candidates) {
            sorted.hidden.push((candidate.fingerprint, reason));
            continue;
        }
        let Ok(info) = &candidate.info else { continue };
        let status = row_status(candidate, info, context);
        let Some(row) = make_row(candidate, status, context.today) else {
            continue;
        };
        match status {
            RowStatus::Usable => sorted.usable.push(row),
            RowStatus::Disabled(_) => sorted.disabled.push(row),
        }
    }
    sort_rows(&mut sorted.usable, context);
    sort_rows(&mut sorted.disabled, context);
    sorted
}

type SortKey = (u8, usize, u8, String, Reverse<i64>);

/// Last used here, then by recency anywhere, then never used: hardware
/// first, name, longest validity.
fn sort_rows(rows: &mut [CertRow], context: &ListContext) {
    rows.sort_by_cached_key(|row| sort_key(row, context));
}

fn sort_key(row: &CertRow, context: &ListContext) -> SortKey {
    let fingerprint = row.candidate.fingerprint;
    if context.last_used_here == Some(fingerprint) {
        return (0, 0, 0, String::new(), Reverse(0));
    }
    if let Some(position) = context
        .recent_anywhere
        .iter()
        .position(|recent| *recent == fingerprint)
    {
        return (1, position, 0, String::new(), Reverse(0));
    }
    let hardware_rank = match row.candidate.hardware {
        Some(true) => 0,
        None => 1,
        Some(false) => 2,
    };
    let not_after = row.candidate.info.as_ref().map_or(0, |info| info.not_after);
    (2, 0, hardware_rank, fold(&row.name), Reverse(not_after))
}

#[cfg(test)]
mod tests;
