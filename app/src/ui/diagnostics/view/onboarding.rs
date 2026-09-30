//! "Getting started" (`docs/ux.md` §8.2), above every tab until the person
//! is ready and has signed the test page, or hides it. It says exactly
//! what is missing, one row per step with the click that fixes it, and the
//! steps already done as check marks; once everything a signature needs is
//! there it turns into "You're ready to sign" with [Test your setup].

use egui::{Align, Frame, Layout, Margin, Stroke, Ui, Vec2};
use websign_i18n::k;
use websign_ui_model::diagnostics::onboarding::{Onboarding, Step, StepState};

use super::Screen;
use super::action::Trailing;
use super::onboarding_steps::{activate_page, test_page, todo};
use super::row::Row;
use super::spans::Span;
use super::words::tr;
use crate::ui::diagnostics::facts::Facts;
use crate::ui::diagnostics::lights::getting_started;
use crate::ui::diagnostics::state::Action;
use crate::ui::icons;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::button::{Button, Kind, Size};
use crate::ui::widgets::list::{self, Position};
use crate::ui::widgets::tone::Tone;
use crate::ui::widgets::{link, text};

/// Space between the tab's header and the checklist.
const ABOVE: f32 = 16.0;

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
    ui.add_space(ABOVE);
    if strip.ready {
        ready(ui, screen, actions);
    } else {
        checklist(ui, screen, facts, &strip, actions);
    }
}

/// The heading line, the steps done, the rows to do and, while the
/// extension has not connected, the link to the site's activation page.
fn checklist(
    ui: &mut Ui,
    screen: &Screen<'_>,
    facts: &Facts,
    strip: &Onboarding,
    actions: &mut Vec<Action>,
) {
    let catalog = screen.catalog;
    let steps = strip.steps();
    let done: Vec<Step> = steps
        .iter()
        .filter(|(_, state)| *state == StepState::Done)
        .map(|(step, _)| *step)
        .collect();
    let progress = catalog
        .tr(k::ONBOARDING_PROGRESS)
        .arg("done", done.len())
        .arg("total", steps.len())
        .to_string();
    heading(ui, screen, &progress, actions);
    if !done.is_empty() {
        ui.add_space(metrics::SPACE_1);
        done_marks(ui, screen, &done);
    }
    ui.add_space(metrics::SPACE_2);
    let to_do: Vec<_> = steps
        .iter()
        .filter(|(_, state)| *state != StepState::Done)
        .map(|(step, state)| todo(catalog, facts, *step, *state))
        .collect();
    list::show(ui, |ui| {
        ui.set_width(ui.available_width());
        for (index, step) in to_do.iter().enumerate() {
            let row = Row {
                id: ui.id().with(("getting-started", index)),
                icon: Some(step.icon),
                title: vec![Span::strong(&step.title)],
                status: Some(step.status.clone()),
                extra: None,
                trailing: step.fix.as_ref().map(|(label, icon, _)| Trailing::Button {
                    label,
                    icon: *icon,
                }),
                position: Position::of(index, to_do.len()),
            };
            if row.show(ui).clicked
                && let Some((_, _, action)) = &step.fix
            {
                actions.push(action.clone());
            }
        }
    });
    if strip.extension != StepState::Done {
        ui.add_space(metrics::SPACE_2);
        activate_link(ui, screen, actions);
    }
}

/// `list-checks` "Getting started" on the left; "2 of 5 done" and [Hide]
/// on the right.
fn heading(ui: &mut Ui, screen: &Screen<'_>, progress: &str, actions: &mut Vec<Action>) {
    let c = theme::colors(ui.ctx());
    let catalog = screen.catalog;
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        text::icon(ui, icons::GETTING_STARTED, metrics::ICON_SM, c.accent_fg);
        let title = tr(catalog, k::ONBOARDING_TITLE);
        let title = ui.label(typography::BODY_STRONG.rich(title).color(c.fg));
        as_heading(ui, &title);
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.spacing_mut().item_spacing.x = metrics::SPACE_2;
            let hide = tr(catalog, k::ONBOARDING_DISMISS);
            if ui.add(Button::ghost(&hide).size(Size::Small)).clicked() {
                actions.push(Action::DismissOnboarding);
            }
            ui.label(typography::SMALL.rich(progress).color(c.fg_muted));
        });
    });
}

/// Screen readers jump between headings: the checklist is one stop.
fn as_heading(ui: &Ui, label: &egui::Response) {
    ui.ctx().accesskit_node_builder(label.id, |node| {
        node.set_role(egui::accesskit::Role::Heading);
        node.set_level(3);
    });
}

/// "✓ App installed  ✓ Certificate found": each mark one label, so a
/// wrapping line never separates an icon from its words.
fn done_marks(ui: &mut Ui, screen: &Screen<'_>, done: &[Step]) {
    let c = theme::colors(ui.ctx());
    let color = Tone::Success.palette(&c).icon;
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing = Vec2::new(metrics::SPACE_4, metrics::SPACE_1);
        for step in done {
            let label = tr(screen.catalog, done_key(*step));
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing.x = 6.0;
                text::icon(ui, icons::SUCCESS, metrics::ICON_SM, color);
                ui.label(typography::SMALL.rich(label).color(c.fg_muted));
            });
        }
    });
}

fn done_key(step: Step) -> websign_i18n::Key {
    match step {
        Step::App => k::ONBOARDING_STEP_APP,
        Step::Extension => k::ONBOARDING_STEP_EXTENSION,
        Step::CardService => k::ONBOARDING_STEP_CARD_SERVICE,
        // A driver step is only listed while it waits.
        Step::Driver => k::DEVICES_SECTION_DRIVERS,
        Step::Certificate => k::ONBOARDING_STEP_CERT,
        Step::TestSignature => k::ONBOARDING_STEP_TEST,
    }
}

/// "Already installed? Finish setting up": the site's page starts the app
/// from the browser and waits until the extension connects.
fn activate_link(ui: &mut Ui, screen: &Screen<'_>, actions: &mut Vec<Action>) {
    let label = tr(screen.catalog, k::ONBOARDING_ACTIVATE);
    let size = Vec2::new(link::width(ui, &label), typography::CAPTION.line_height);
    let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
    let id = ui.id().with("getting-started.activate");
    if link::paint(ui, id, rect.min, &label).clicked() {
        actions.push(Action::OpenUrl(activate_page()));
    }
}

/// Everything a signature needs is there: say so, and offer the test page
/// once.
fn ready(ui: &mut Ui, screen: &Screen<'_>, actions: &mut Vec<Action>) {
    let c = theme::colors(ui.ctx());
    let catalog = screen.catalog;
    let palette = Tone::Success.palette(&c);
    Frame::new()
        .fill(c.bg_surface)
        .stroke(Stroke::new(1.0, c.border))
        .corner_radius(metrics::RADIUS_LG)
        .inner_margin(Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(metrics::SPACE_3, 2.0);
                text::icon(ui, icons::SUCCESS, metrics::ICON_MD, palette.icon);
                ui.vertical(|ui| {
                    let title = tr(catalog, k::ONBOARDING_READY_TITLE);
                    let body = tr(catalog, k::ONBOARDING_READY_BODY);
                    let title = ui.label(typography::BODY_STRONG.rich(title).color(c.fg));
                    as_heading(ui, &title);
                    ui.add(
                        egui::Label::new(typography::SMALL.rich(body).color(c.fg_muted)).wrap(),
                    );
                    ui.add_space(metrics::SPACE_3 - 2.0);
                    ui.horizontal(|ui| {
                        ui.spacing_mut().item_spacing.x = metrics::SPACE_2;
                        let test = tr(catalog, k::ONBOARDING_TEST_BUTTON);
                        let button = Button::new(Kind::Primary, &test)
                            .icon(icons::EXTERNAL_LINK)
                            .size(Size::Small);
                        if ui.add(button).clicked() {
                            actions.push(Action::OpenUrl(test_page()));
                        }
                        let hide = tr(catalog, k::ONBOARDING_DISMISS);
                        if ui.add(Button::ghost(&hide).size(Size::Small)).clicked() {
                            actions.push(Action::DismissOnboarding);
                        }
                    });
                });
            });
        });
}
