//! The PIN area of the window (`docs/ux.md` §4.6).

use super::machine::{ConfirmModel, ConfirmState};
use super::view::{PinBlock, PinError, PinSystem};
use crate::certs::{
    CertCandidate, CertRow, DeviceLabel, DisabledReason, KeySource, PinMode, RowStatus,
};

impl ConfirmModel {
    /// What the PIN area shows for the selected certificate.
    pub(super) fn pin_block(&self) -> PinBlock {
        let signing = self.state == ConfirmState::Signing;
        let hidden = matches!(
            self.state,
            ConfirmState::Idle | ConfirmState::LoadingCerts | ConfirmState::Empty
        );
        let Some(row) = self.selected_row().filter(|_| !hidden) else {
            return PinBlock::Hidden;
        };
        if self
            .mode()
            .is_none_or(|mode| matches!(mode, super::port::Mode::Choose))
        {
            return PinBlock::Hidden;
        }
        match row.status {
            RowStatus::Disabled(DisabledReason::PinLocked) => PinBlock::Locked,
            RowStatus::Disabled(_) => PinBlock::Hidden,
            RowStatus::Usable => match self.path_of(&row.candidate) {
                (source, PinMode::System) => match pin_system(source) {
                    Some(system) => PinBlock::OsPrompt {
                        now: signing,
                        system,
                    },
                    // A driver never shows a dialog of its own: C_Login
                    // needs the PIN from us.
                    None => self.field(row, None),
                },
                (_, PinMode::PinPad) => PinBlock::PinPad { now: signing },
                (_, PinMode::Unlocked) => PinBlock::Unlocked,
                (_, PinMode::App { length, .. }) => self.field(row, length),
            },
        }
    }

    /// The path signing takes (`via`) and who asks for the
    /// PIN on it: after "Try through the token driver" the PIN area follows
    /// the driver, not the store that failed. A path the candidate does not
    /// have falls back to the primary one.
    pub(super) fn path_of<'a>(&self, candidate: &'a CertCandidate) -> (&'a KeySource, PinMode) {
        match self
            .via
            .checked_sub(1)
            .and_then(|index| candidate.alternates.get(index))
        {
            Some(path) => (&path.source, path.pin),
            None => (&candidate.source, candidate.pin),
        }
    }

    fn field(&self, row: &CertRow, length: Option<(u32, u32)>) -> PinBlock {
        PinBlock::Field {
            card: matches!(row.candidate.device, Some(DeviceLabel::CardInReader { .. })),
            length,
            valid: self.pin_length_valid(length),
            error: self.pin_error,
        }
    }

    /// Whether our PIN field, if shown, holds a PIN of acceptable length.
    pub(super) fn pin_acceptable(&self) -> bool {
        match self.pin_block() {
            PinBlock::Field { valid, .. } => valid,
            _ => true,
        }
    }

    fn pin_length_valid(&self, length: Option<(u32, u32)>) -> bool {
        let typed = u32::try_from(self.pin_len).unwrap_or(u32::MAX);
        match length {
            Some((min, max)) => (min..=max).contains(&typed),
            None => typed > 0,
        }
    }
}

/// Whose dialog asks for the PIN of a key reached through `source`; `None`
/// for a token driver, which has none.
fn pin_system(source: &KeySource) -> Option<PinSystem> {
    match source {
        KeySource::Windows => Some(PinSystem::Windows),
        KeySource::MacosKeychain | KeySource::MacosToken => Some(PinSystem::Macos),
        KeySource::Driver { .. } => None,
    }
}

/// The error line the field shows after a wrong PIN.
pub(super) fn pin_error_of(count_low: bool, final_try: bool) -> PinError {
    if final_try {
        PinError::IncorrectFinal
    } else if count_low {
        PinError::IncorrectLow
    } else {
        PinError::Incorrect
    }
}
