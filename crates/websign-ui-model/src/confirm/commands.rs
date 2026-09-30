//! Host commands: what the window does when the engine speaks.

use std::time::{Duration, Instant};

use super::machine::{ConfirmModel, ConfirmState};
use super::port::{OpenRequest, UiCommand};
use super::slot::CodeSlot;
use crate::certs::{CertCandidate, CertList, ListContext, RowStatus, build_cert_list};
use crate::possible::PossibleCard;

/// Applies `command`. Commands for a request that is not on screen are stale
/// (the caller moved on) and are dropped.
pub(super) fn apply(model: &mut ConfirmModel, command: UiCommand, now: Instant) {
    match command {
        UiCommand::Open(request) => model.open(request, now),
        UiCommand::Hide => model.reset(),
        UiCommand::Certificates {
            key,
            candidates,
            possible,
            context,
        } if model.is_current(key) => model.certificates(&candidates, possible, &context, now),
        UiCommand::DigestPending { key, fingerprint }
            if model.is_current(key) && model.state == ConfirmState::Choosing =>
        {
            // Before Continue the window has not released the certificate,
            // so no digest can be pending for it (D11).
            let selected = model.list.as_ref().and_then(|list| list.selected);
            if selected == Some(fingerprint) && model.code != CodeSlot::Hint {
                model.code = CodeSlot::Preparing(now);
            }
        }
        UiCommand::DigestReady {
            key,
            fingerprint,
            code,
        } if model.is_current(key) => {
            // Only a digest the window is waiting for counts: one for a
            // certificate no longer selected is stale, and one before a new
            // caller pressed Continue was never asked for (D11).
            let selected = model.list.as_ref().and_then(|list| list.selected);
            let waiting = matches!(model.state, ConfirmState::Choosing | ConfirmState::Ready)
                && model.code != CodeSlot::Hint;
            if waiting && selected == Some(fingerprint) {
                model.code = CodeSlot::Ready(code);
                model.state = ConfirmState::Ready;
                model.rearm(now);
            }
        }
        UiCommand::Signing { key } if model.is_current(key) => model.signing_started(),
        UiCommand::Failed { key, failure } if model.is_current(key) => model.failed(failure),
        UiCommand::Finished { key, finish } if model.is_current(key) => model.finished(finish, now),
        UiCommand::Queue { key, position } if model.is_current(key) => {
            if let Some(request) = &mut model.request {
                request.position = position;
            }
        }
        _ => {}
    }
}

impl ConfirmModel {
    fn open(&mut self, request: OpenRequest, now: Instant) {
        self.reset();
        self.deadline = Some(now + Duration::from_secs(u64::from(request.timeout_secs)));
        self.request = Some(request);
        self.state = ConfirmState::LoadingCerts;
        // The next request of a queue re-arms a focused window at once; a
        // window that is only being shown waits for `Focus(true)`.
        self.rearm(now);
    }

    /// A fresh listing. While searching (loading or empty) the list is built
    /// from scratch; once rows are on screen it is merged, so nothing moves
    /// under the pointer (`docs/ux.md` §5.9). Any row that becomes able to
    /// sign (a token inserted or plugged back in) re-arms the button, because
    /// what sits under the pointer changed (§4.7).
    fn certificates(
        &mut self,
        candidates: &[CertCandidate],
        possible: Vec<PossibleCard>,
        context: &ListContext,
        now: Instant,
    ) {
        if !self.on_screen() {
            return;
        }
        self.possible = possible;
        let searching = matches!(self.state, ConfirmState::LoadingCerts | ConfirmState::Empty);
        let before = self.list.as_ref().map_or(0, usable_count);
        let list = match self.list.take() {
            Some(mut list) if !searching => {
                list.append(candidates, context);
                list
            }
            _ => build_cert_list(candidates, context),
        };
        let after = usable_count(&list);
        let has_selection = list.selected.is_some();
        self.list = Some(list);
        if searching {
            if has_selection {
                self.enter_choosing(now);
            } else {
                self.state = ConfirmState::Empty;
            }
        } else if after > before {
            self.rearm(now);
        }
    }

    fn signing_started(&mut self) {
        if matches!(
            self.state,
            ConfirmState::Ready | ConfirmState::PinError | ConfirmState::Error { .. }
        ) {
            self.state = ConfirmState::Signing;
        }
    }
}

/// Rows that can sign right now, wherever they sit.
fn usable_count(list: &CertList) -> usize {
    list.usable
        .iter()
        .filter(|row| row.status == RowStatus::Usable)
        .count()
}
