//! A listing as a flow sees it: the candidates, the list the window will
//! build from them, and which certificates may be chosen.

use websign_core::Fingerprint;
use websign_ui_model::certs::{CertCandidate, CertList, ListContext, RowStatus, build_cert_list};
use websign_ui_model::confirm::UiCommand;
use websign_ui_model::confirm::port::RequestKey;

use crate::ports::KeySnapshot;

/// The snapshot and the list rules applied to it. The rules are the
/// window's own, so what the host accepts is exactly what the window offers.
#[derive(Debug)]
pub(super) struct Listing {
    snapshot: KeySnapshot,
    list: CertList,
    context: ListContext,
}

impl Listing {
    pub fn build(snapshot: &KeySnapshot, context: ListContext) -> Listing {
        Listing {
            list: build_cert_list(&snapshot.candidates, &context),
            snapshot: snapshot.clone(),
            context,
        }
    }

    pub fn snapshot(&self) -> &KeySnapshot {
        &self.snapshot
    }

    /// The certificate the list preselects.
    pub fn preselected(&self) -> Option<Fingerprint> {
        self.list.selected
    }

    /// The candidate for `fingerprint`, when it is listed and can sign.
    pub fn usable(&self, fingerprint: &Fingerprint) -> Option<&CertCandidate> {
        self.list
            .usable
            .iter()
            .find(|row| {
                row.status == RowStatus::Usable && row.candidate.fingerprint == *fingerprint
            })
            .map(|row| &row.candidate)
    }

    /// Any candidate for `fingerprint`, usable or not.
    pub fn candidate(&self, fingerprint: &Fingerprint) -> Option<&CertCandidate> {
        self.snapshot
            .candidates
            .iter()
            .find(|candidate| candidate.fingerprint == *fingerprint)
    }

    /// The window's copy of the listing.
    pub fn command(&self, key: RequestKey) -> UiCommand {
        UiCommand::Certificates {
            key,
            candidates: self.snapshot.candidates.clone(),
            possible: self.snapshot.possible.clone(),
            context: self.context.clone(),
        }
    }
}
