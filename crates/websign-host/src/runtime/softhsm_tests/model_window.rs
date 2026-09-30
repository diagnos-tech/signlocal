//! A window made of the real `ConfirmModel`, pressing what a person would
//! press: the decisions, the PIN area and the paths are the model's own, so
//! the engine and the model are tested together.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use secrecy::SecretString;
use websign_core::Fingerprint;
use websign_ui_model::confirm::port::{Failure, RequestKey};
use websign_ui_model::confirm::view::{PinBlock, PrimaryButton};
use websign_ui_model::confirm::{
    ConfirmModel, ConfirmState, Intent, UiCommand, UiEvent, UserInput,
};

use crate::engine::EngineEvent;
use crate::ports::ConfirmUi;
use crate::runtime::EventSender;

/// Each `Sign` the window sent: its path, and whether it carried a PIN.
pub(super) type Signs = Arc<Mutex<Vec<(usize, bool)>>>;

pub(super) struct ModelWindow {
    pub events: EventSender,
    pub target: Fingerprint,
    pub pin: String,
    pub signs: Signs,
    pub model: ConfirmModel,
    /// Time as the model sees it: a second passes before every step, so
    /// every button is armed when pressed.
    pub now: Instant,
    pub key: Option<RequestKey>,
}

impl ConfirmUi for ModelWindow {
    fn command(&mut self, command: UiCommand) {
        let opened = matches!(&command, UiCommand::Open(_));
        if let UiCommand::Open(request) = &command {
            self.key = Some(request.key);
        }
        self.model.apply(command, self.now);
        if opened {
            let _ = self.model.input(UserInput::Focus(true), self.now);
        }
        // Each step may lead to the next (the alternate path shows the PIN
        // field, then Sign); a few are enough for any state.
        for _ in 0..4 {
            if !self.step() {
                break;
            }
        }
    }

    fn parent_window(&self) -> Option<isize> {
        None
    }
}

impl ModelWindow {
    /// Presses the next thing a person would; `false` when there is none.
    fn step(&mut self) -> bool {
        self.now += Duration::from_secs(1);
        let view = self.model.view(self.now);
        let alternate = matches!(
            view.banner,
            Some(Failure::DriverFailure {
                alternate: true,
                ..
            })
        );
        let intents = match (self.model.state(), view.footer.primary) {
            (ConfirmState::Choosing, _) if view.selected != Some(self.target) => {
                self.input(UserInput::Select(self.target))
            }
            (ConfirmState::Error { .. }, _) if alternate => self.input(UserInput::UseAlternatePath),
            (_, PrimaryButton::Continue | PrimaryButton::Sign) if view.footer.primary_enabled => {
                self.click()
            }
            (ConfirmState::Ready, _) if matches!(view.pin, PinBlock::Field { .. }) => {
                self.input(UserInput::PinLength(self.pin.chars().count()))
            }
            _ => return false,
        };
        let pin_field = matches!(self.model.view(self.now).pin, PinBlock::Field { .. });
        for intent in intents {
            self.send(intent, pin_field);
        }
        true
    }

    fn input(&mut self, input: UserInput) -> Vec<Intent> {
        self.model.input(input, self.now)
    }

    fn click(&mut self) -> Vec<Intent> {
        let mut intents = self.input(UserInput::PrimaryPress);
        self.now += Duration::from_millis(10);
        intents.extend(self.input(UserInput::PrimaryRelease));
        intents
    }

    /// As the app's outbox: the PIN goes only with a `Sign` while our field
    /// is on screen.
    fn send(&mut self, intent: Intent, pin_field: bool) {
        let Some(key) = self.key else {
            return;
        };
        let event = match intent {
            Intent::Selected(fingerprint) => UiEvent::Selected { key, fingerprint },
            Intent::Continue(fingerprint) => UiEvent::Continue { key, fingerprint },
            Intent::Sign {
                fingerprint, via, ..
            } => {
                self.signs.lock().unwrap().push((via, pin_field));
                UiEvent::Sign {
                    key,
                    fingerprint,
                    via,
                    pin: pin_field.then(|| SecretString::from(self.pin.clone())),
                    remember: false,
                }
            }
            other => panic!("unexpected decision {other:?}"),
        };
        self.events.send(EngineEvent::Ui(event)).unwrap();
    }
}
