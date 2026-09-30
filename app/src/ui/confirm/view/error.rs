//! The error notice (`docs/ux.md` §4.8, §15): title and text on a danger
//! background, the actions the failure offers, and a collapsed "Technical
//! details" with the copyable native code. "Try again" itself is the
//! primary button in the footer.

use egui::{Ui, Vec2};
use websign_i18n::k;
use websign_ui_model::confirm::UserInput;
use websign_ui_model::confirm::port::Failure;

use super::{Action, Screen};
use crate::ui::confirm::failure_text::failure_text;
use crate::ui::icons;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::banner::Banner;
use crate::ui::widgets::button::{Button, Size};
use crate::ui::widgets::tone::Tone;

pub fn show(ui: &mut Ui, s: &mut Screen<'_>, failure: &Failure) {
    let text = failure_text(s.tr, failure);
    ui.add(Banner::new(Tone::Danger, &text.body).title(&text.title));
    let alternate = matches!(
        failure,
        Failure::DriverFailure {
            alternate: true,
            ..
        }
    );
    let diagnostics = matches!(
        failure,
        Failure::DriverFailure { .. } | Failure::Internal { .. }
    );
    if !alternate && !diagnostics && text.technical.is_none() {
        return;
    }
    let tr = s.tr;
    ui.add_space(metrics::SPACE_2);
    // `horizontal_wrapped`, not a wrapping layout: that one would claim
    // the whole height left in the body and push the list off screen.
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(metrics::SPACE_2);
        if alternate {
            let label = tr.tr(k::ACTION_TRY_DRIVER).to_string();
            if Button::secondary(&label)
                .icon(icons::DRIVER)
                .show(ui)
                .response
                .clicked()
            {
                s.input(UserInput::UseAlternatePath);
            }
        }
        if let (Failure::Internal { .. }, Some(detail)) = (failure, &text.technical) {
            let label = tr.tr(k::COMMON_COPY_DETAILS).to_string();
            if Button::secondary(&label)
                .icon(icons::COPY)
                .show(ui)
                .response
                .clicked()
            {
                s.out
                    .push(Action::Copy(format!("{}: {detail}", text.title)));
            }
        }
        if diagnostics {
            let label = tr.tr(k::COMMON_OPEN_DIAGNOSTICS).to_string();
            if Button::ghost(&label)
                .icon(icons::DIAGNOSTICS)
                .show(ui)
                .response
                .clicked()
            {
                s.input(UserInput::OpenDiagnostics);
            }
        }
    });
    if let Some(detail) = &text.technical {
        technical(ui, s, detail);
    }
}

/// "Technical details", collapsed; open, the code in mono and "Copy".
fn technical(ui: &mut Ui, s: &mut Screen<'_>, detail: &str) {
    let c = theme::colors(ui.ctx());
    let label = s.tr.tr(k::COMMON_TECHNICAL_DETAILS).to_string();
    let icon = if s.session.technical_open {
        icons::COLLAPSE
    } else {
        icons::EXPAND
    };
    let toggle = Button::ghost(&label).icon(icon).size(Size::Small).show(ui);
    if toggle.response.clicked() {
        s.session.technical_open = !s.session.technical_open;
    }
    if !s.session.technical_open {
        return;
    }
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing = Vec2::splat(metrics::SPACE_2);
        ui.add(egui::Label::new(typography::MONO.rich(detail).color(c.fg_muted)).selectable(true));
        let copy = s.tr.tr(k::COMMON_COPY).to_string();
        if Button::ghost(&copy)
            .icon(icons::COPY)
            .size(Size::Small)
            .show(ui)
            .response
            .clicked()
        {
            s.out.push(Action::Copy(detail.to_owned()));
        }
    });
}
