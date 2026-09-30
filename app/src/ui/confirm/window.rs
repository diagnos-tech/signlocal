//! The window's state and its frame: host commands in, person input
//! through the model, decisions out. No eframe here, so tests drive it with
//! `egui_kittest` exactly as the real event loop does ([`super::run`]).

use std::time::{Duration, Instant};

use egui::{Key, Ui};
use websign_host::runtime::EventSender;
use websign_i18n::Catalog;
use websign_ui_model::confirm::port::Failure;
use websign_ui_model::confirm::view::PinBlock;
use websign_ui_model::confirm::{ConfirmModel, ConfirmState, ConfirmView, UiCommand, UserInput};

use super::clock::Clock;
use super::session::{FocusTarget, Session};
use super::view::{self, Action, Screen};
use super::{guard, outbox, words};
use crate::ui::theme::Installed;

/// The eframe app state of the confirmation window.
#[derive(Debug)]
pub struct ConfirmWindow {
    model: ConfirmModel,
    session: Session,
    tr: Catalog,
    events: EventSender,
    clock: Clock,
    /// The focus last reported to the model.
    focused: Option<bool>,
}

impl ConfirmWindow {
    /// A hidden window. `Installed` proves fonts and themes are in place.
    pub fn new(_installed: Installed, tr: Catalog, events: EventSender, clock: Clock) -> Self {
        ConfirmWindow {
            model: ConfirmModel::new(),
            session: Session::default(),
            tr,
            events,
            clock,
            focused: None,
        }
    }

    /// Applies a host command.
    pub fn apply(&mut self, command: UiCommand) {
        let now = self.clock.now();
        match &command {
            UiCommand::Open(request) => self.session.start(request.key),
            UiCommand::Failed { key, failure } if self.session.key == Some(*key) => {
                // The model forgets the typed length on a wrong PIN; the
                // field is emptied with it and gets the focus back. Other
                // failures ask for another choice: focus goes to the list.
                self.session.clear_pin();
                self.session.focus = Some(match failure {
                    Failure::PinIncorrect { .. } => FocusTarget::Pin,
                    _ => FocusTarget::SelectedRow,
                });
            }
            UiCommand::Hide | UiCommand::Finished { .. } => self.session.clear_pin(),
            _ => {}
        }
        let before = self.model.state().clone();
        self.model.apply(command, now);
        self.after_change(&before, now);
    }

    /// Nothing on screen: the window should be hidden.
    pub fn is_idle(&self) -> bool {
        *self.model.state() == ConfirmState::Idle
    }

    /// A result notice is on screen until its hold ends.
    pub fn is_holding(&self) -> bool {
        matches!(
            self.model.state(),
            ConfirmState::Success | ConfirmState::SiteCancelled | ConfirmState::Timeout
        )
    }

    /// The OS close button: Cancel (`docs/ux.md` §4.1).
    pub fn close_button(&mut self, ctx: &egui::Context) {
        let now = self.clock.now();
        self.input(ctx, UserInput::CloseButton, now);
    }

    /// Characters in the PIN field (tests check that stray keys never land).
    #[cfg(test)]
    pub fn typed_pin_len(&self) -> usize {
        self.session.pin.chars().count()
    }

    /// What the window shows now.
    pub fn view(&self) -> ConfirmView {
        self.model.view(self.clock.now())
    }

    /// The OS title for the request on screen.
    pub fn title(&self) -> Option<String> {
        (!self.is_idle()).then(|| words::window_title(&self.tr, &self.view()))
    }

    /// Advances time-driven changes (holds ending, arming) and says when
    /// the next one is due.
    pub fn tick(&mut self) -> Option<Instant> {
        let now = self.clock.now();
        let before = self.model.state().clone();
        let _ = self.model.tick(now);
        self.after_change(&before, now);
        self.model.next_deadline(now)
    }

    /// One frame: input guard, focus, drawing, then the person's input.
    pub fn frame(&mut self, ui: &mut Ui) {
        let next = self.tick();
        let now = self.clock.now();
        self.track_focus(ui.input(|input| input.focused), now);
        if !self.is_idle() && !self.model.is_armed(now) {
            let navigation = self.model.accepts_selection(now);
            ui.input_mut(|input| guard::drop_keystrokes(input, navigation));
        }
        let escape = ui.input(|input| input.key_pressed(Key::Escape));
        let view = self.model.view(now);
        let mut actions = Vec::new();
        if !self.is_idle() {
            view::show(
                ui,
                Screen {
                    view: &view,
                    tr: &self.tr,
                    session: &mut self.session,
                    out: &mut actions,
                    now,
                },
            );
        }
        if escape {
            actions.push(Action::Input(UserInput::Escape));
        }
        for action in actions {
            self.act(ui.ctx(), action, now);
        }
        if let Some(at) = next {
            ui.ctx().request_repaint_after(
                at.saturating_duration_since(now) + Duration::from_millis(1),
            );
        }
    }

    fn track_focus(&mut self, focused: bool, now: Instant) {
        if self.focused != Some(focused) {
            self.focused = Some(focused);
            let _ = self.model.input(UserInput::Focus(focused), now);
        }
    }

    fn act(&mut self, ctx: &egui::Context, action: Action, now: Instant) {
        match action {
            Action::Input(input) => self.input(ctx, input, now),
            Action::OpenUrl(url) => {
                let _ = crate::platform::system_ui::open_url(&url);
            }
            Action::Copy(text) => ctx.copy_text(text),
        }
    }

    fn input(&mut self, ctx: &egui::Context, input: UserInput, now: Instant) {
        let before = self.model.state().clone();
        let intents = self.model.input(input, now);
        let pin_field = matches!(self.model.view(now).pin, PinBlock::Field { .. });
        if let Some(key) = self.session.key {
            for intent in intents {
                outbox::send(&self.events, key, intent, &mut self.session, pin_field);
            }
        }
        // The PIN buffer may have been wiped (sent, or a new selection).
        let _ = self
            .model
            .input(UserInput::PinLength(self.session.pin.chars().count()), now);
        // "Try through the token driver" may need our PIN first: the field
        // it brings takes the focus.
        let error = matches!(before, ConfirmState::Error { .. });
        if error && *self.model.state() == ConfirmState::Ready && pin_field {
            self.session.focus = Some(FocusTarget::Pin);
        }
        self.after_change(&before, now);
        ctx.request_repaint();
    }

    /// Housekeeping when the state moved: first placement of focus, and
    /// forgetting the request once the window goes idle.
    fn after_change(&mut self, before: &ConfirmState, now: Instant) {
        let state = self.model.state().clone();
        if state == ConfirmState::Idle && *before != ConfirmState::Idle {
            self.session.end();
            return;
        }
        if !self.session.placed && matches!(state, ConfirmState::Choosing | ConfirmState::Empty) {
            self.session.placed = true;
            self.session.focus = Some(match (&state, self.model.view(now).pin) {
                (ConfirmState::Empty, _) => FocusTarget::Cancel,
                (_, PinBlock::Field { .. }) => FocusTarget::Pin,
                _ => FocusTarget::SelectedRow,
            });
        }
    }
}
