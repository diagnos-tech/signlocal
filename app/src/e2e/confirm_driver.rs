//! The confirmation window's e2e driver: screenshots of every state it
//! shows, and the person's part (`WEBSIGN_E2E_CONFIRM`) as accessibility
//! actions on the real widgets, so arming, the D11 Continue step and the
//! PIN field all run as they do for a person.

use std::collections::VecDeque;
use std::time::{Duration, Instant};

use egui::accesskit::{Action, Role};
use egui::{Context, Event, FullOutput, RawInput, Ui};
use websign_i18n::k;
use websign_ui_model::confirm::UserInput;

use super::capture::Capture;
use super::tree::{self, Widgets};
use super::{E2eConfig, shadow};

/// How long a state stays on screen before its screenshot (the arming
/// animation ends at 600 ms).
const SETTLE: Duration = Duration::from_millis(300);
const SETTLE_READY: Duration = Duration::from_millis(900);
/// Result notices close by themselves (success after 900 ms): no wait.
const SETTLE_NOTICE: Duration = Duration::from_millis(30);
/// States too short to save reliably (a software key signs in
/// milliseconds; waiting for its picture would cost the success notice
/// after it). The kittest snapshots cover them.
const UNSAVED: &[&str] = &["signing"];
/// A person cannot act before the window arms (600 ms after it shows and
/// has focus); typing earlier would even be dropped by its keystroke guard.
const ARMED: Duration = Duration::from_millis(700);
/// Pause between two actions of the driver.
const PACE: Duration = Duration::from_millis(250);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Part {
    /// Press the primary button at every step.
    Confirm,
    /// Tick "Remember", then as `Confirm`.
    Remember,
    Cancel,
    /// Leave the decision to the page.
    Wait,
}

/// Button and checkbox names, in the window's language.
#[derive(Debug)]
struct Labels {
    primary: Vec<String>,
    cancel: Vec<String>,
    remember: Vec<String>,
}

impl Labels {
    fn load() -> Labels {
        let tr = crate::ui::i18n::catalog();
        let text =
            |keys: &[websign_i18n::Key]| keys.iter().map(|key| tr.tr(*key).to_string()).collect();
        Labels {
            primary: text(&[k::ACTION_SIGN, k::ACTION_CONTINUE, k::ACTION_USE_CERT]),
            cancel: text(&[k::COMMON_CANCEL]),
            remember: text(&[k::CONSENT_REMEMBER, k::CONSENT_REMEMBER_APP]),
        }
    }
}

/// The egui plugin.
#[derive(Debug)]
pub struct ConfirmDriver {
    part: Part,
    pin: Option<String>,
    capture: Option<Capture>,
    labels: Labels,
    widgets: Widgets,
    /// The state on screen and since when.
    state: Option<(String, Instant)>,
    /// States already saved as screenshots.
    shot: Vec<String>,
    pin_typed: bool,
    last_action: Option<Instant>,
    /// Input for the coming frames, one batch per frame.
    queued: VecDeque<Vec<Event>>,
}

impl ConfirmDriver {
    pub fn new(config: E2eConfig) -> ConfirmDriver {
        let part = match config.confirm.as_deref() {
            Some("sign" | "choose") => Part::Confirm,
            Some("remember") => Part::Remember,
            Some("cancel") => Part::Cancel,
            _ => Part::Wait,
        };
        ConfirmDriver {
            part,
            pin: config.pin,
            capture: config.screenshots.map(|dir| Capture::new(dir, "confirm")),
            labels: Labels::load(),
            widgets: Widgets::default(),
            state: None,
            shot: Vec::new(),
            pin_typed: false,
            last_action: None,
            queued: VecDeque::new(),
        }
    }

    /// Follows the state on screen; returns it with how long it has shown.
    fn follow(&mut self) -> Option<(String, Duration)> {
        let now = shadow::state_name();
        if self.state.as_ref().map(|(name, _)| name) != now.as_ref() {
            // The field starts empty with each request (the next one may
            // follow a result notice at once) and is emptied after a wrong
            // PIN; otherwise it keeps what was typed.
            let fresh = [
                "loading",
                "pin-error",
                "success",
                "site-cancelled",
                "timeout",
            ];
            if now.as_deref().is_none_or(|name| fresh.contains(&name)) {
                self.pin_typed = false;
            }
            log::debug!(
                "e2e: the window shows {}",
                now.as_deref().unwrap_or("nothing")
            );
            self.state = now.map(|name| (name, Instant::now()));
        }
        self.state
            .as_ref()
            .map(|(name, since)| (name.clone(), since.elapsed()))
    }

    fn screenshot_due(&self, name: &str, shown: Duration) -> bool {
        let settle = match name {
            "ready" => SETTLE_READY,
            "success" | "site-cancelled" | "timeout" => SETTLE_NOTICE,
            _ => SETTLE,
        };
        self.capture.is_some()
            && shown >= settle
            && !UNSAVED.contains(&name)
            && !self.shot.iter().any(|s| s == name)
    }

    /// The person's next step, if any is due.
    fn act(&mut self) {
        if self.part == Part::Wait || self.last_action.is_some_and(|at| at.elapsed() < PACE) {
            return;
        }
        let press = vec![UserInput::PrimaryPress, UserInput::PrimaryRelease];
        let w = &self.widgets;
        let step = match self.part {
            Part::Cancel => w
                .find(Role::Button, &self.labels.cancel)
                .filter(|b| b.enabled)
                .map(|b| (b.id, None, vec![UserInput::CancelButton])),
            _ => {
                let remember = w.find(Role::CheckBox, &self.labels.remember);
                let pin = w.first(Role::PasswordInput);
                match (remember, pin, &self.pin) {
                    (Some(b), _, _) if self.part == Part::Remember && b.enabled && !b.checked => {
                        Some((b.id, None, vec![UserInput::Remember(true)]))
                    }
                    (_, Some(field), Some(pin)) if !self.pin_typed && field.enabled => {
                        let typed = UserInput::PinLength(pin.chars().count());
                        Some((field.id, Some(pin.clone()), vec![typed]))
                    }
                    _ => w
                        .find(Role::Button, &self.labels.primary)
                        .filter(|b| b.enabled)
                        .map(|b| (b.id, None, press)),
                }
            }
        };
        let Some((target, text, mirrored)) = step else {
            return;
        };
        self.last_action = Some(Instant::now());
        log::debug!("e2e: {mirrored:?}");
        let mut now = RawInput::default();
        match text {
            Some(text) => {
                self.pin_typed = true;
                tree::request(&mut now, target, Action::Focus);
                self.queued.push_back(now.events);
                self.queued.push_back(vec![Event::Text(text)]);
            }
            None => {
                tree::request(&mut now, target, Action::Click);
                self.queued.push_back(now.events);
            }
        }
        shadow::mirror(&mirrored);
    }
}

impl egui::Plugin for ConfirmDriver {
    fn debug_name(&self) -> &'static str {
        "websign-e2e-confirm"
    }

    fn input_hook(&mut self, ctx: &Context, input: &mut RawInput) {
        let shown = shadow::state_name();
        if let Some(capture) = &mut self.capture {
            if let Some(dropped) = capture.abandon_unless(ctx, shown.as_deref()) {
                self.shot.retain(|s| *s != dropped);
            }
            capture.input(ctx, input);
        }
        if self.part != Part::Wait {
            // Shown means focused, hidden means not: the focus cycle a
            // window manager gives each request (the model arms on it).
            input.focused = shown.is_some();
        }
        if let Some(events) = self.queued.pop_front() {
            input.events.extend(events);
        }
    }

    fn output_hook(&mut self, _ctx: &Context, output: &mut FullOutput) {
        self.widgets
            .update(output.platform_output.accesskit_update.as_ref());
    }

    fn on_end_pass(&mut self, ui: &mut Ui) {
        let ctx = ui.ctx().clone();
        let busy = self.capture.as_mut().is_some_and(|capture| {
            capture.end_pass(&ctx);
            capture.is_busy()
        });
        let Some((name, shown)) = self.follow() else {
            return;
        };
        ctx.request_repaint_after(PACE);
        if busy || !self.queued.is_empty() {
            return;
        }
        if self.screenshot_due(&name, shown) {
            if let Some(capture) = &mut self.capture {
                capture.start(&ctx, &name);
            }
            self.shot.push(name);
            return;
        }
        let waiting_for_shot = self.capture.is_some()
            && !UNSAVED.contains(&name.as_str())
            && !self.shot.contains(&name);
        if !waiting_for_shot && shown >= ARMED {
            self.act();
        }
    }
}
