//! What the person does: clicks, keys, selection, closing (`docs/ux.md`
//! §4.7, §4.9).

use std::time::Instant;

use websign_core::Fingerprint;

use super::cancel::cancel_code;
use super::machine::{ConfirmModel, ConfirmState, Intent, UserInput};
use super::view::PinBlock;
use crate::certs::RowStatus;

/// Applies `input`.
///
/// Leaving is never gated: Esc, Cancel and the close button work at any
/// moment, armed or not, because a person must always be able to say no.
/// Arming only guards decisions that grant something (Continue, Sign, Use
/// this certificate, Remember) and the other clicks and keys of the window,
/// so a keystroke or double-click meant for the site never lands here
/// (`docs/ux.md` §4.7). Focus changes and PIN typing are always taken: they
/// decide nothing. Selection is not approval: once the window has been
/// armed since it gained focus, `Select` is taken during the re-arm it
/// causes, so the arrows move freely while Sign waits its 600 ms again.
pub(super) fn handle(model: &mut ConfirmModel, input: UserInput, now: Instant) -> Vec<Intent> {
    let armed = model.arming.is_armed(now);
    if armed {
        model.settled = true;
    }
    match input {
        UserInput::Focus(focused) => {
            model.focused = focused;
            model.settled = false;
            if focused {
                model.arming.rearm(now);
            } else {
                model.arming.disarm();
            }
        }
        UserInput::PinLength(length) => model.pin_len = length,
        UserInput::PrimaryPress => model.arming.press(now),
        UserInput::PrimaryRelease => {
            if model.arming.release(now) {
                return model.activate(now);
            }
        }
        UserInput::Escape | UserInput::CancelButton | UserInput::CloseButton => {
            return model.close();
        }
        UserInput::Enter if armed => return model.enter(),
        UserInput::Select(fingerprint) if model.settled => return model.select(fingerprint, now),
        UserInput::Remember(checked) if armed => model.set_remember(checked),
        UserInput::Rescan if armed && model.on_screen() => return model.rescan(now),
        UserInput::OpenDiagnostics if armed && model.on_screen() => {
            return vec![Intent::OpenDiagnostics(None)];
        }
        UserInput::UseAlternatePath if armed => return model.use_alternate_path(),
        _ => {}
    }
    Vec::new()
}

impl ConfirmModel {
    fn set_remember(&mut self, checked: bool) {
        if self
            .request
            .as_ref()
            .is_some_and(|request| !request.remembered && request.can_remember)
        {
            self.remember = checked;
        }
    }

    fn select(&mut self, fingerprint: Fingerprint, now: Instant) -> Vec<Intent> {
        let changing = matches!(
            self.state,
            ConfirmState::Choosing
                | ConfirmState::Ready
                | ConfirmState::PinError
                | ConfirmState::PinLocked
                | ConfirmState::Error { .. }
        );
        let Some(list) = self.list.as_mut().filter(|_| changing) else {
            return Vec::new();
        };
        let usable = list
            .usable
            .iter()
            .any(|row| row.candidate.fingerprint == fingerprint && row.status == RowStatus::Usable);
        if !usable || list.selected == Some(fingerprint) {
            return Vec::new();
        }
        list.selected = Some(fingerprint);
        self.enter_choosing(now);
        self.pin_len = 0;
        self.pin_error = None;
        self.banner = None;
        self.via = 0;
        self.rearm(now);
        vec![Intent::Selected(fingerprint)]
    }

    fn rescan(&mut self, now: Instant) -> Vec<Intent> {
        if self.state == ConfirmState::Empty {
            self.state = ConfirmState::LoadingCerts;
            self.list = None;
            self.slow_device = None;
            self.loading_since = Some(now);
        }
        vec![Intent::Rescan]
    }

    /// Esc, Cancel or the close button.
    ///
    /// Every press while a request waits sends a cancel, even a repeated one:
    /// the host drops a cancel for a request that already ended, and a
    /// second press must never be swallowed if the first was lost. On a
    /// result screen the request is already answered, so closing only hides
    /// the notice. While the OS or a PIN pad signs, nothing can abort the
    /// operation (their dialogs have their own Cancel), so the key is ignored
    /// and the button reads "Please wait…".
    fn close(&mut self) -> Vec<Intent> {
        match self.state {
            ConfirmState::Idle => Vec::new(),
            ConfirmState::Success | ConfirmState::SiteCancelled | ConfirmState::Timeout => {
                self.reset();
                Vec::new()
            }
            ConfirmState::Signing if !self.cancel_allowed() => Vec::new(),
            _ => vec![Intent::Cancel(cancel_code(&self.state))],
        }
    }

    /// Neither the OS nor a PIN pad can be aborted once signing started.
    pub(super) fn cancel_allowed(&self) -> bool {
        !(self.state == ConfirmState::Signing
            && matches!(
                self.pin_block(),
                PinBlock::OsPrompt { .. } | PinBlock::PinPad { .. }
            ))
    }
}
