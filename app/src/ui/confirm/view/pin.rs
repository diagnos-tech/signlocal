//! The PIN area (`docs/ux.md` §4.6): our field for keys behind a token
//! driver, the keypad note, "Token unlocked for this session", or the
//! locked notice in the field's place. System prompts only show in the
//! footer hint.

use egui::{Id, Response, Sense, Ui, Vec2};
use websign_i18n::k;
use websign_ui_model::confirm::view::{PinBlock, PinError};
use websign_ui_model::confirm::{ConfirmState, ConfirmView, UserInput};

use super::Screen;
use crate::ui::confirm::failure_text::failure_text;
use crate::ui::confirm::session::FocusTarget;
use crate::ui::fonts::{self, Face, Weight};
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, typography};
use crate::ui::widgets::banner::Banner;
use crate::ui::widgets::chip::Chip;
use crate::ui::widgets::pin_field::PinField;
use crate::ui::widgets::text;
use crate::ui::widgets::tone::Tone;

const GAP: f32 = 6.0;
/// Icons of the message lines under the field.
const ICON: f32 = 14.0;
/// Without a stated maximum the field still stops somewhere.
const MAX_PIN: u32 = 64;

/// The id of our PIN field (tests find it by name; the window focuses it).
pub fn field_id() -> Id {
    Id::new("confirm.pin")
}

pub fn visible(view: &ConfirmView) -> bool {
    matches!(
        view.pin,
        PinBlock::Field { .. }
            | PinBlock::PinPad { now: false }
            | PinBlock::Unlocked
            | PinBlock::Locked
    )
}

/// Draws the PIN area; returns our field, when it shows one.
pub fn show(ui: &mut Ui, s: &mut Screen<'_>) -> Option<Response> {
    match s.view.pin {
        PinBlock::Field {
            card,
            length,
            error,
            ..
        } => return Some(field(ui, s, card, length, error)),
        PinBlock::PinPad { now: false } => {
            let note = s.tr.tr(k::PIN_PINPAD_BEFORE).to_string();
            line(
                ui,
                icons::PIN_PAD,
                &note,
                theme::colors(ui.ctx()).fg_muted,
                false,
            );
        }
        PinBlock::Unlocked => {
            let label = s.tr.tr(k::PIN_UNLOCKED_SESSION).to_string();
            ui.add(Chip::new(Tone::Neutral, &label).icon(icons::TOKEN_UNLOCKED));
        }
        PinBlock::Locked => {
            let title = s.tr.tr(k::PIN_LOCKED_TITLE).to_string();
            let body = match &s.view.banner {
                Some(failure) => failure_text(s.tr, failure).body,
                None => s.tr.tr(k::PIN_LOCKED_BODY_GENERIC).to_string(),
            };
            ui.add(
                Banner::new(Tone::Danger, &body)
                    .title(&title)
                    .icon(icons::PIN_LOCKED),
            );
        }
        PinBlock::Hidden | PinBlock::OsPrompt { .. } | PinBlock::PinPad { .. } => {}
    }
    None
}

fn field(
    ui: &mut Ui,
    s: &mut Screen<'_>,
    card: bool,
    length: Option<(u32, u32)>,
    error: Option<PinError>,
) -> Response {
    let c = theme::colors(ui.ctx());
    let tr = s.tr;
    let name = tr
        .tr(if card {
            k::PIN_LABEL_CARD
        } else {
            k::PIN_LABEL_TOKEN
        })
        .to_string();
    ui.horizontal(|ui| {
        ui.label(typography::CAPTION.rich(&name).color(c.fg_muted));
        if let Some((min, max)) = length {
            let hint = tr
                .tr(k::PIN_LENGTH_HINT)
                .arg("min", min)
                .arg("max", max)
                .to_string();
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.label(typography::SMALL.rich(hint).color(c.fg_subtle));
            });
        }
    });
    ui.add_space(GAP);
    let (show, hide) = (
        tr.tr(k::PIN_SHOW).to_string(),
        tr.tr(k::PIN_HIDE).to_string(),
    );
    let max = length.map_or(MAX_PIN, |(_, max)| max.min(MAX_PIN));
    let shown = PinField {
        pin: &mut s.session.pin,
        shown: &mut s.session.pin_shown,
        id: field_id(),
        name: &name,
        show_label: &show,
        hide_label: &hide,
        max_chars: usize::try_from(max).unwrap_or(usize::MAX),
        error: error.is_some(),
        enabled: s.view.state != ConfirmState::Signing,
    }
    .show(ui);
    if s.session.fit.placed() && s.session.take_focus(FocusTarget::Pin) {
        shown.field.request_focus();
    }
    if shown.changed {
        let typed = s.session.pin.chars().count();
        s.input(UserInput::PinLength(typed));
    }
    if shown.submitted {
        s.input(UserInput::Enter);
    }
    ui.add_space(GAP);
    let (text_key, strong) = match error {
        Some(PinError::Incorrect) => (Some(k::PIN_INCORRECT), false),
        Some(PinError::IncorrectLow) => (Some(k::PIN_INCORRECT_LOW), false),
        Some(PinError::IncorrectFinal) => (Some(k::PIN_INCORRECT_FINAL), true),
        None => (None, false),
    };
    match text_key {
        Some(key) => {
            let message = tr.tr(key).to_string();
            line(ui, icons::ERROR, &message, c.danger, strong);
        }
        None => {
            let privacy = tr.tr(k::PIN_PRIVACY).to_string();
            line(ui, icons::PRIVACY, &privacy, c.fg_subtle, false);
        }
    }
    shown.field
}

/// An icon and a wrapping line of `text-small` (600 for the last try). An
/// error line is announced at once (§14: assertive).
fn line(ui: &mut Ui, icon: Icon, message: &str, color: egui::Color32, strong: bool) {
    ui.horizontal_top(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(GAP, 0.0);
        let glyph = text::whole(ui, icon.rich(ICON, color));
        let (rect, _) = ui.allocate_exact_size(
            Vec2::new(ICON, typography::SMALL.line_height),
            Sense::hover(),
        );
        ui.painter()
            .galley(rect.center() - glyph.size() / 2.0, glyph, color);
        let weight = if strong {
            Weight::SemiBold
        } else {
            Weight::Regular
        };
        let rich = typography::SMALL
            .rich(message)
            .family(fonts::family(Face::Ui, weight))
            .color(color);
        let label = ui.add(egui::Label::new(rich).wrap());
        if color == theme::colors(ui.ctx()).danger {
            ui.ctx().accesskit_node_builder(label.id, |node| {
                node.set_live(egui::accesskit::Live::Assertive);
            });
        }
    });
}
