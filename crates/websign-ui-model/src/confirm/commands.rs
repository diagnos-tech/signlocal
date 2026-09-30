//! Host commands: what the window does when the engine speaks.

use std::time::{Duration, Instant};

use super::machine::{ConfirmModel, ConfirmState};
use super::port::{OpenRequest, UiCommand};
use super::slot::CodeSlot;
use crate::certs::{CertCandidate, ListContext, RowStatus, build_cert_list};
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
            if model.list.as_ref().and_then(|list| list.selected) == Some(fingerprint) {
                model.code = CodeSlot::Preparing(now);
            }
        }
        UiCommand::DigestReady {
            key,
            fingerprint,
            code,
        } if model.is_current(key) => {
            let selected = model.list.as_ref().and_then(|list| list.selected);
            let waiting = matches!(model.state, ConfirmState::Choosing | ConfirmState::Ready);
            if waiting && selected == Some(fingerprint) {
                model.code = CodeSlot::Ready(code);
                model.state = ConfirmState::Ready;
                model.arming.rearm(now);
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
        self.arming.rearm(now);
    }

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
        let before = self.list.as_ref().map_or(0, |list| list.usable.len());
        let list = match self.list.take() {
            Some(mut list) if !searching => {
                list.append(candidates, context);
                list
            }
            _ => build_cert_list(candidates, context),
        };
        let list = self.list.insert(list);
        let first_usable = list
            .usable
            .iter()
            .find(|row| row.status == RowStatus::Usable)
            .map(|row| row.candidate.fingerprint);
        if list.selected.is_none() {
            list.selected = first_usable;
        }
        let grew = list.usable.len() > before;
        if searching {
            if first_usable.is_some() {
                self.enter_choosing(now);
            } else {
                self.state = ConfirmState::Empty;
            }
        } else if grew {
            self.arming.rearm(now);
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
