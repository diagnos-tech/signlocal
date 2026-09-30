//! Describing the window at one instant (`docs/ux.md` §4.2–§4.11).

use std::time::Instant;

use websign_core::present::caller::CallerLabel;

use super::machine::{ConfirmModel, ConfirmState};
use super::outcome::is_retryable;
use super::port::{CallerView, Mode};
use super::slot::{CodeSlot, SKELETON_DELAY};
use super::timers::seconds_left;
use super::view::{
    CodeCard, ConfirmView, FooterHint, FooterView, HeaderView, PinBlock, PrimaryButton, RememberBox,
};

/// The view of `model` at `now`.
pub(super) fn view(model: &ConfirmModel, now: Instant) -> ConfirmView {
    let pin = model.pin_block();
    let (primary, primary_enabled) = primary(model, now);
    let seconds = model
        .on_screen()
        .then(|| seconds_left(model, now))
        .flatten();
    ConfirmView {
        state: model.state.clone(),
        mode: model.mode().unwrap_or(Mode::Choose),
        header: header(model),
        code: code_card(model, now),
        list: model.list.clone(),
        selected: model.list.as_ref().and_then(|list| list.selected),
        possible: model.possible.clone(),
        pin,
        remember: remember_box(model),
        banner: match model.state {
            ConfirmState::Error { .. } | ConfirmState::PinLocked => model.banner.clone(),
            _ => None,
        },
        footer: FooterView {
            hint: hint(model, pin, seconds),
            primary,
            primary_enabled,
            cancel_enabled: model.cancel_allowed(),
            primary_first: cfg!(target_os = "windows"),
        },
    }
}

/// The header of the request on screen. An idle window is hidden and never
/// rendered, so its header is a blank desktop caller rather than an
/// `Option` every renderer would have to unwrap.
fn header(model: &ConfirmModel) -> HeaderView {
    match &model.request {
        Some(request) => HeaderView {
            caller: request.caller.clone(),
            remembered: request.remembered,
            queue: (request.position.1 > 1).then_some(request.position),
        },
        None => HeaderView {
            caller: CallerView::Desktop {
                label: CallerLabel {
                    name: String::new(),
                    detail: String::new(),
                    verified: false,
                },
            },
            remembered: false,
            queue: None,
        },
    }
}

fn code_card(model: &ConfirmModel, now: Instant) -> CodeCard {
    let Some(mode) = model.mode() else {
        return CodeCard::Hidden;
    };
    if model.selected_row().is_none() {
        return CodeCard::Hidden;
    }
    match (mode, &model.code) {
        (Mode::Choose, _) => CodeCard::SelectShares,
        (Mode::Sign { .. }, CodeSlot::Hint) => CodeCard::ContinueHint,
        (Mode::Sign { .. }, CodeSlot::Preparing(since)) => CodeCard::Preparing {
            skeleton: now.saturating_duration_since(*since) >= SKELETON_DELAY,
        },
        (Mode::Sign { hash }, CodeSlot::Ready(code)) => CodeCard::Ready {
            code: code.clone(),
            hash,
        },
        (Mode::Sign { .. }, CodeSlot::None) => CodeCard::Hidden,
    }
}

fn remember_box(model: &ConfirmModel) -> RememberBox {
    match &model.request {
        Some(request) if request.remembered => RememberBox::Hidden,
        Some(request) if !request.can_remember => RememberBox::Disabled,
        Some(_) => RememberBox::Enabled {
            checked: model.remember,
        },
        None => RememberBox::Hidden,
    }
}

/// Left text of the footer: the countdown wins over any next-step hint.
fn hint(model: &ConfirmModel, pin: PinBlock, seconds: Option<u32>) -> FooterHint {
    if let Some(seconds) = seconds {
        return FooterHint::ExpiresIn { seconds };
    }
    match (&model.state, pin) {
        (ConfirmState::Empty, _) => FooterHint::OpenDiagnostics,
        (_, PinBlock::OsPrompt { .. }) => FooterHint::OsPinPrompt,
        _ => FooterHint::None,
    }
}

/// The primary button's label, and whether a click would count now.
fn primary(model: &ConfirmModel, now: Instant) -> (PrimaryButton, bool) {
    let armed = model.arming.is_armed(now);
    let usable = model.selected_usable().is_some();
    let Some(mode) = model.mode() else {
        return (PrimaryButton::Sign, false);
    };
    match (mode, &model.state) {
        (Mode::Choose, state) => (
            PrimaryButton::UseCertificate,
            *state == ConfirmState::Choosing && armed && usable && !model.chosen,
        ),
        (Mode::Sign { .. }, ConfirmState::Signing) => (PrimaryButton::Signing, false),
        (Mode::Sign { .. }, ConfirmState::Choosing) if model.code == CodeSlot::Hint => {
            (PrimaryButton::Continue, armed && usable)
        }
        (Mode::Sign { .. }, ConfirmState::Ready | ConfirmState::PinError) => {
            (PrimaryButton::Sign, armed && model.can_sign())
        }
        (Mode::Sign { .. }, ConfirmState::Error { .. })
            if model.banner.as_ref().is_some_and(is_retryable) =>
        {
            (PrimaryButton::Retry, armed && model.can_sign())
        }
        _ => (PrimaryButton::Sign, false),
    }
}
