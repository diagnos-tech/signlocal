//! "Getting started" (`docs/ux.md` §8.2): a strip above every tab until all
//! four steps are done or the person hides it. "Test a signature" opens the
//! test page, which signs 32 random bytes and shows the same verification
//! code.

use egui::text::LayoutJob;
use egui::{Align, Color32, FontSelection, Frame, Label, Margin, RichText, Stroke, Ui};
use websign_i18n::k;
use websign_ui_model::diagnostics::onboarding::StepState;

use super::Screen;
use super::words::tr;
use crate::ui::diagnostics::facts::Facts;
use crate::ui::diagnostics::lights::getting_started;
use crate::ui::diagnostics::state::Action;
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::button::{Button, Kind, Size};
use crate::ui::widgets::tone::Tone;

/// The test page. TODO(gustavo): publish it on the site (`/test`).
pub fn test_page() -> String {
    format!("{}test/", websign_project::HOMEPAGE)
}

pub fn show(ui: &mut Ui, screen: &Screen<'_>, facts: &Facts, actions: &mut Vec<Action>) {
    let settings = screen.settings;
    let strip = getting_started(
        facts,
        settings.onboarding_dismissed,
        settings.test_signature_done,
    );
    if !strip.visible {
        return;
    }
    let c = theme::colors(ui.ctx());
    let catalog = screen.catalog;
    ui.add_space(14.0);
    Frame::new()
        .fill(c.bg_surface)
        .stroke(Stroke::new(1.0, c.border))
        .corner_radius(metrics::RADIUS_LG)
        .inner_margin(Margin::symmetric(14, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            // Rows as tall as the text, not as a button.
            ui.spacing_mut().interact_size.y = typography::SMALL.line_height;
            ui.horizontal_wrapped(|ui| {
                ui.spacing_mut().item_spacing = egui::vec2(14.0, 8.0);
                let title = tr(catalog, k::ONBOARDING_TITLE);
                let title = typography::CAPTION.rich(title).color(c.fg);
                ui.add(labelled(icons::GETTING_STARTED, c.accent_fg, title));
                let steps = [
                    (strip.app, k::ONBOARDING_STEP_APP),
                    (strip.extension, k::ONBOARDING_STEP_EXTENSION),
                    (strip.certificate, k::ONBOARDING_STEP_CERT),
                    (strip.test_signature, k::ONBOARDING_STEP_TEST),
                ];
                for (state, key) in steps {
                    step(ui, state, &tr(catalog, key));
                }
            });
            ui.add_space(metrics::SPACE_3);
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = metrics::SPACE_2;
                let test = tr(catalog, k::ONBOARDING_TEST_BUTTON);
                let button = Button::new(Kind::Primary, &test).size(Size::Small);
                if strip.test_signature != StepState::Done && ui.add(button).clicked() {
                    actions.push(Action::OpenUrl(test_page()));
                }
                let hide = tr(catalog, k::ONBOARDING_DISMISS);
                if ui.add(Button::ghost(&hide).size(Size::Small)).clicked() {
                    actions.push(Action::DismissOnboarding);
                }
            });
        });
}

fn step(ui: &mut Ui, state: StepState, label: &str) {
    let c = theme::colors(ui.ctx());
    let (icon, color) = match state {
        StepState::Done => (icons::SUCCESS, Tone::Success.palette(&c).icon),
        StepState::Attention => (icons::ATTENTION, Tone::Warning.palette(&c).icon),
        StepState::Pending => (icons::NOT_APPLICABLE, c.fg_subtle),
    };
    ui.add(labelled(
        icon,
        color,
        typography::SMALL.rich(label).color(c.fg_muted),
    ));
}

/// An icon and its text as one label, so a wrapping line never separates
/// them.
fn labelled(icon: Icon, color: Color32, text: RichText) -> Label {
    let style = egui::Style::default();
    let mut job = LayoutJob::default();
    icon.rich(metrics::ICON_SM, color).append_to(
        &mut job,
        &style,
        FontSelection::Default,
        Align::Center,
    );
    text.append_to(&mut job, &style, FontSelection::Default, Align::Center);
    if let Some(section) = job.sections.get_mut(1) {
        section.leading_space = 6.0;
    }
    Label::new(job).extend()
}
