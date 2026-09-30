//! What the person does: clicks, keys, selection, closing (`docs/ux.md`
//! §4.7, §4.9).

use std::time::Instant;

use websign_core::Fingerprint;

use super::cancel::cancel_code;
use super::machine::{ConfirmModel, ConfirmState, Intent, UserInput};
use super::outcome::is_retryable;
use super::port::{Failure, Mode};
use super::slot::CodeSlot;
use super::view::PinBlock;
use crate::certs::RowStatus;

/// Applies `input`. Everything except Esc, focus changes and PIN typing is
/// discarded while the window is unarmed, so a keystroke or click meant for
/// the site never becomes a decision here.
pub(super) fn handle(model: &mut ConfirmModel, input: UserInput, now: Instant) -> Vec<Intent> {
    let armed = model.arming.is_armed(now);
    match input {
        UserInput::Focus(true) => model.arming.rearm(now),
        UserInput::Focus(false) => model.arming.disarm(),
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
        UserInput::Enter if armed => return model.activate(now),
        UserInput::Select(fingerprint) if armed => return model.select(fingerprint, now),
        UserInput::Remember(checked) if armed => model.set_remember(checked),
        UserInput::Rescan if armed && model.on_screen() => return model.rescan(),
        UserInput::OpenDiagnostics if armed && model.on_screen() => {
            return vec![Intent::OpenDiagnostics(None)];
        }
        UserInput::UseAlternatePath if armed => return model.use_alternate_path(),
        _ => {}
    }
    Vec::new()
}

impl ConfirmModel {
    /// Enter or a valid click on the primary button.
    fn activate(&mut self, now: Instant) -> Vec<Intent> {
        let (Some(mode), Some(fingerprint)) = (self.mode(), self.selected_usable()) else {
            return Vec::new();
        };
        match (mode, &self.state) {
            (Mode::Choose, ConfirmState::Choosing) if !self.chosen => {
                self.chosen = true;
                vec![Intent::Choose {
                    fingerprint,
                    remember: self.remember_wanted(),
                }]
            }
            (Mode::Sign { .. }, ConfirmState::Choosing) if self.code == CodeSlot::Hint => {
                self.code = CodeSlot::Preparing(now);
                vec![Intent::Continue(fingerprint)]
            }
            (Mode::Sign { .. }, ConfirmState::Ready | ConfirmState::PinError)
                if self.can_sign() =>
            {
                self.begin_signing(fingerprint)
            }
            (Mode::Sign { .. }, ConfirmState::Error { .. })
                if self.banner.as_ref().is_some_and(is_retryable) && self.can_sign() =>
            {
                self.begin_signing(fingerprint)
            }
            _ => Vec::new(),
        }
    }

    fn begin_signing(&mut self, fingerprint: Fingerprint) -> Vec<Intent> {
        self.state = ConfirmState::Signing;
        self.pin_error = None;
        self.banner = None;
        vec![Intent::Sign {
            fingerprint,
            via: self.via,
            remember: self.remember_wanted(),
        }]
    }

    /// "Remember this site" only counts for a caller that could still be
    /// remembered.
    fn remember_wanted(&self) -> bool {
        self.request
            .as_ref()
            .is_some_and(|request| self.remember && !request.remembered && request.can_remember)
    }

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
        self.arming.rearm(now);
        vec![Intent::Selected(fingerprint)]
    }

    /// "Try through the token driver": sign again over the first alternate path.
    fn use_alternate_path(&mut self) -> Vec<Intent> {
        let offered = matches!(
            self.banner,
            Some(Failure::DriverFailure {
                alternate: true,
                ..
            })
        );
        let Some(fingerprint) = self
            .selected_usable()
            .filter(|_| offered && self.can_sign())
        else {
            return Vec::new();
        };
        self.via = 1;
        self.begin_signing(fingerprint)
    }

    fn rescan(&mut self) -> Vec<Intent> {
        if self.state == ConfirmState::Empty {
            self.state = ConfirmState::LoadingCerts;
            self.list = None;
        }
        vec![Intent::Rescan]
    }

    /// Esc, Cancel or the close button.
    fn close(&mut self) -> Vec<Intent> {
        match self.state {
            ConfirmState::Idle => Vec::new(),
            ConfirmState::Success | ConfirmState::SiteCancelled | ConfirmState::Timeout => {
                self.reset();
                Vec::new()
            }
            ConfirmState::Signing if !self.cancel_allowed() => Vec::new(),
            _ if self.cancel_sent => Vec::new(),
            _ => {
                self.cancel_sent = true;
                vec![Intent::Cancel(cancel_code(&self.state))]
            }
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
