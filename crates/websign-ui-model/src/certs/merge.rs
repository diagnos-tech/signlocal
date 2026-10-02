//! Merging a fresh listing into an open window's list (`docs/ux.md` §4.8,
//! "Live changes").

use super::build_row::make_row;
use super::candidate::CertCandidate;
use super::order::{ListContext, classify};
use super::row::{CertList, CertRow, DisabledReason, RowStatus};
use super::status::row_status;

impl CertList {
    /// Merges a fresh listing into an open window's list: rows keep their
    /// place; new usable rows go to the end of the usable group; rows whose
    /// token left become `Disabled(Removed)` in place; returning tokens
    /// re-enable their rows. A selection never moves by itself; a list with
    /// no selection (nothing was usable) selects its first usable row, as a
    /// fresh build would, since no choice of the person is overridden.
    ///
    /// Rows missing from `candidates` are kept as they are: a listing that
    /// skips a certificate is a key store hiccup, while a token that left is
    /// reported with `removed`. Dropping rows would shift the list under the
    /// pointer (`docs/ux.md` §5.9).
    pub fn append(&mut self, candidates: &[CertCandidate], context: &ListContext) {
        for row in self.usable.iter_mut().chain(self.disabled.iter_mut()) {
            if let Some(fresh) = candidates
                .iter()
                .find(|c| c.fingerprint == row.candidate.fingerprint)
            {
                refresh(row, fresh, context);
            }
        }
        self.promote_returned();
        let known = |fingerprint: &_| {
            self.usable
                .iter()
                .chain(&self.disabled)
                .any(|row| row.candidate.fingerprint == *fingerprint)
                || self.hidden.iter().any(|(hidden, _)| hidden == fingerprint)
        };
        let new = classify(candidates, context, known);
        self.usable.extend(new.usable);
        self.disabled.extend(new.disabled);
        self.hidden.extend(new.hidden);
        if self.selected.is_none() {
            self.selected = self
                .usable
                .iter()
                .find(|row| row.status == RowStatus::Usable)
                .map(|row| row.candidate.fingerprint);
        }
    }

    /// Rows that were disabled for another reason and are usable again go
    /// to the end of the usable group.
    fn promote_returned(&mut self) {
        let mut index = 0;
        while index < self.disabled.len() {
            if self.disabled[index].status == RowStatus::Usable {
                let row = self.disabled.remove(index);
                self.usable.push(row);
            } else {
                index += 1;
            }
        }
    }
}

/// Updates a row from the fresh candidate. Only `Removed` is reversible: a
/// locked PIN or an expiry does not clear by itself while the window is open.
fn refresh(row: &mut CertRow, fresh: &CertCandidate, context: &ListContext) {
    if fresh.removed {
        row.status = RowStatus::Disabled(DisabledReason::Removed);
        row.candidate.removed = true;
        return;
    }
    if row.status != RowStatus::Disabled(DisabledReason::Removed) {
        return;
    }
    let Ok(info) = &fresh.info else { return };
    let status = row_status(fresh, info, context);
    if let Some(updated) = make_row(fresh, status, context) {
        *row = updated;
    }
}
