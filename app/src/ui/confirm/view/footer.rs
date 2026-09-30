//! The footer (`docs/ux.md` §4.2, §4.7): the next-step hint on the left,
//! Cancel and the primary button on the right, in the platform's order
//! (Windows: primary first).
//!
//! Buttons are created in visual order, left to right, so Tab follows what
//! the eye sees (§4.9); that is why their widths are measured first.
//!
//! The primary button reports its halves separately: the model counts a
//! click only when the press **and** the release come after arming, and a
//! re-arm between them cancels it (§4.7). Space or an assistive-technology
//! action on the focused button arrives as a whole click and is reported as
//! press and release together.

use egui::{Align, Layout, Rect, Ui, UiBuilder, Vec2, pos2};
use websign_i18n::k;
use websign_ui_model::confirm::view::{FooterHint, PinBlock, PrimaryButton};
use websign_ui_model::confirm::{ConfirmState, UserInput};

use super::Screen;
use crate::ui::confirm::session::FocusTarget;
use crate::ui::confirm::words;
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::button::{Button, Size};
use crate::ui::widgets::{link, text};

const BUTTON_GAP: f32 = 8.0;
const HINT_GAP: f32 = 12.0;
/// `widgets::button` pads footer buttons by 16 on each side.
const PADDING: f32 = 16.0;

pub fn show(ui: &mut Ui, s: &mut Screen<'_>) {
    let rect = ui.max_rect();
    if matches!(
        s.view.state,
        ConfirmState::Success | ConfirmState::SiteCancelled | ConfirmState::Timeout
    ) {
        let close = s.tr.tr(k::COMMON_CLOSE).to_string();
        let width = button_width(ui, &close, false, metrics::CONTROL_MD);
        let area = right_aligned(rect, width);
        let clicked = in_row(ui, area, |ui| {
            Button::secondary(&close)
                .size(Size::Large)
                .show(ui)
                .response
                .clicked()
        });
        if clicked {
            s.input(UserInput::CloseButton);
        }
        return;
    }
    let footer = s.view.footer;
    let cancel = if footer.cancel_enabled {
        s.tr.tr(k::COMMON_CANCEL)
    } else {
        s.tr.tr(k::COMMON_WAIT)
    }
    .to_string();
    let primary = primary_label(s, footer.primary);
    let busy = footer.primary == PrimaryButton::Signing;
    let widths = (
        button_width(ui, &cancel, false, metrics::CONTROL_MD),
        button_width(ui, &primary, busy, metrics::PRIMARY_MIN_WIDTH),
    );
    let total = widths.0 + widths.1 + BUTTON_GAP;
    hint(
        ui,
        s,
        Rect::from_min_max(rect.min, pos2(rect.max.x - total - HINT_GAP, rect.max.y)),
    );
    let area = right_aligned(rect, total);
    in_row(ui, area, |ui| {
        ui.spacing_mut().item_spacing = Vec2::new(BUTTON_GAP, 0.0);
        if footer.primary_first {
            primary_button(ui, s, &primary, busy);
            cancel_button(ui, s, &cancel);
        } else {
            cancel_button(ui, s, &cancel);
            primary_button(ui, s, &primary, busy);
        }
    });
}

fn primary_label(s: &Screen<'_>, button: PrimaryButton) -> String {
    let key = match button {
        PrimaryButton::Sign => k::ACTION_SIGN,
        PrimaryButton::Signing => k::ACTION_SIGNING,
        PrimaryButton::Retry => k::COMMON_RETRY,
        PrimaryButton::Continue => k::ACTION_CONTINUE,
        PrimaryButton::UseCertificate => k::ACTION_USE_CERT,
    };
    s.tr.tr(key).to_string()
}

fn primary_button(ui: &mut Ui, s: &mut Screen<'_>, label: &str, busy: bool) {
    let shown = Button::primary(label)
        .enabled(s.view.footer.primary_enabled || busy)
        .busy(busy)
        .show(ui);
    let response = &shown.response;
    if s.session.take_focus(FocusTarget::Primary) {
        response.request_focus();
    }
    let pressed = ui.input(|input| input.pointer.primary_pressed());
    let released = ui.input(|input| input.pointer.primary_released());
    if pressed && response.contains_pointer() && response.sense.senses_click() {
        s.input(UserInput::PrimaryPress);
    }
    if response.clicked() {
        if !released {
            s.input(UserInput::PrimaryPress);
        }
        s.input(UserInput::PrimaryRelease);
    }
    if shown.enter {
        s.input(UserInput::Enter);
    }
}

fn cancel_button(ui: &mut Ui, s: &mut Screen<'_>, label: &str) {
    let shown = Button::secondary(label)
        .size(Size::Large)
        .enabled(s.view.footer.cancel_enabled)
        .show(ui);
    if s.session.take_focus(FocusTarget::Cancel) {
        shown.response.request_focus();
    }
    if shown.response.clicked() || shown.enter {
        s.input(UserInput::CancelButton);
    }
}

/// The next-step hint: countdown, OS or keypad PIN prompt, or "Open
/// diagnostics". The OS named is the key's store (`PinSystem`), never the
/// running system: a token driver (all of Linux) has no system dialog.
fn hint(ui: &mut Ui, s: &mut Screen<'_>, rect: Rect) {
    let c = theme::colors(ui.ctx());
    let tr = s.tr;
    let (icon, text) = match (s.view.footer.hint, s.view.pin) {
        (FooterHint::ExpiresIn { seconds }, _) => (
            None,
            tr.tr(k::FOOTER_EXPIRES_IN)
                .arg("seconds", seconds)
                .to_string(),
        ),
        (FooterHint::OpenDiagnostics, _) => {
            let label = tr.tr(k::COMMON_OPEN_DIAGNOSTICS).to_string();
            let glyph = text::whole(ui, icons::DIAGNOSTICS.rich(metrics::ICON_SM, c.accent_fg));
            let top = rect.center().y - glyph.size().y / 2.0;
            ui.painter()
                .galley(pos2(rect.min.x, top), glyph, c.accent_fg);
            let at = pos2(rect.min.x + metrics::ICON_SM + 6.0, rect.center().y - 8.0);
            if link::paint(ui, ui.id().with("diagnostics"), at, &label).clicked() {
                s.input(UserInput::OpenDiagnostics);
            }
            return;
        }
        (FooterHint::OsPinPrompt, PinBlock::OsPrompt { now, system }) => {
            let key = if now {
                k::PIN_OS_PROMPT_NOW
            } else {
                k::PIN_OS_PROMPT
            };
            let text = tr.tr(key).arg("os", words::pin_system(tr, system));
            (now.then_some(icons::PIN), text.to_string())
        }
        // The model pairs the OS hint with an OS prompt; anything else has
        // no dialog to announce.
        (FooterHint::OsPinPrompt, _) => return,
        (FooterHint::None, PinBlock::PinPad { now: true }) => {
            (Some(icons::PIN_PAD), tr.tr(k::PIN_PINPAD_NOW).to_string())
        }
        (FooterHint::None, _) => return,
    };
    paint_hint(ui, rect, icon, &text);
}

fn paint_hint(ui: &mut Ui, rect: Rect, icon: Option<Icon>, hint: &str) {
    let c = theme::colors(ui.ctx());
    let mut x = rect.min.x;
    if let Some(icon) = icon {
        let glyph = text::whole(ui, icon.rich(metrics::ICON_SM, c.accent_fg));
        let top = rect.center().y - glyph.size().y / 2.0;
        ui.painter().galley(pos2(x, top), glyph, c.accent_fg);
        x += metrics::ICON_SM + 6.0;
    }
    let area = Rect::from_min_max(pos2(x, rect.min.y), rect.max);
    let galley = text::wrapped(
        ui,
        typography::SMALL.rich(hint).color(c.fg_muted),
        area.width(),
    );
    let top = rect.center().y - galley.size().y / 2.0;
    let bounds = Rect::from_min_size(pos2(x, top), galley.size());
    let response = ui.interact(bounds, ui.id().with("hint"), egui::Sense::hover());
    response.widget_info(|| egui::WidgetInfo::labeled(egui::WidgetType::Label, true, hint));
    ui.painter().galley(bounds.min, galley, c.fg_muted);
}

/// The width `widgets::button` gives a footer button labeled `label`.
fn button_width(ui: &Ui, label: &str, busy: bool, min: f32) -> f32 {
    let text = text::whole(ui, typography::BUTTON.rich(label)).size().x;
    let lead = if busy {
        metrics::ICON_SM + metrics::SPACE_2
    } else {
        0.0
    };
    (text + lead + 2.0 * PADDING).max(min)
}

fn right_aligned(rect: Rect, width: f32) -> Rect {
    Rect::from_min_max(pos2(rect.max.x - width, rect.min.y), rect.max)
}

fn in_row<R>(ui: &mut Ui, area: Rect, add: impl FnOnce(&mut Ui) -> R) -> R {
    ui.scope_builder(
        UiBuilder::new()
            .max_rect(area)
            .layout(Layout::left_to_right(Align::Center)),
        add,
    )
    .inner
}
