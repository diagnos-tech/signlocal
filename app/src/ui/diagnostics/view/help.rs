//! Help tab (`docs/ux.md` §8.6): common questions, "Report a problem" with
//! the exact text "Copy diagnostics" copies, and About.

use egui::accesskit::Role;
use egui::{Frame, Label, Margin, ScrollArea, Stroke, Ui};
use websign_i18n::k;

use super::Screen;
use super::faq;
use super::section;
use super::words::tr;
use crate::ui::diagnostics::state::Action;
use crate::ui::icons;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::button::Button;

/// Eight lines of the preview (`max-height: 150px` with padding).
const PREVIEW_HEIGHT: f32 = 8.0 * 16.0;

/// Where people report problems.
pub fn support_page() -> String {
    format!("{}/issues", websign_project::REPOSITORY)
}

fn privacy_page() -> String {
    format!("{}privacy.html", websign_project::HOMEPAGE)
}

pub fn show(ui: &mut Ui, screen: &Screen<'_>, actions: &mut Vec<Action>) {
    let catalog = screen.catalog;
    section::label(ui, &tr(catalog, k::HELP_FAQ_TITLE), None);
    faq::show(ui, catalog, &screen.state.faq_open, actions);

    section::label(ui, &tr(catalog, k::HELP_REPORT_TITLE), None);
    preview(ui, screen);
    ui.add_space(metrics::SPACE_2);
    section::note(ui, &tr(catalog, k::HELP_REPORT_PREVIEW));
    ui.add_space(metrics::SPACE_3);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = metrics::SPACE_3;
        let copy = tr(catalog, k::DIAG_COPY);
        let button = Button::secondary(&copy)
            .icon(icons::COPY)
            .enabled(screen.report.is_some());
        if ui.add(button).clicked() {
            actions.push(Action::CopyReport);
        }
        let support = tr(catalog, k::HELP_OPEN_SUPPORT);
        if ui
            .add(Button::ghost(&support).icon(icons::EXTERNAL_LINK))
            .clicked()
        {
            actions.push(Action::OpenUrl(support_page()));
        }
    });

    section::label(ui, &tr(catalog, k::HELP_ABOUT_TITLE), None);
    about(ui, screen, actions);
}

fn preview(ui: &mut Ui, screen: &Screen<'_>) {
    let c = theme::colors(ui.ctx());
    let text = screen.report.unwrap_or_default().trim_end().to_owned();
    let name = tr(screen.catalog, k::HELP_REPORT_TITLE);
    Frame::new()
        .fill(c.bg_sunken)
        .stroke(Stroke::new(1.0, c.border))
        .corner_radius(metrics::RADIUS_MD)
        .inner_margin(Margin::symmetric(12, 10))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ScrollArea::both()
                .id_salt("report-preview")
                .max_height(PREVIEW_HEIGHT)
                // Nested in the tab's scroll area, the available height is
                // what is left on screen: keep eight lines anyway.
                .min_scrolled_height(PREVIEW_HEIGHT)
                .auto_shrink([false, true])
                .show(ui, |ui| {
                    let response =
                        ui.add(Label::new(typography::MONO.rich(&text).color(c.fg)).extend());
                    ui.ctx().accesskit_node_builder(response.id, |node| {
                        node.set_role(Role::Document);
                        node.set_label(name);
                    });
                });
        });
}

fn about(ui: &mut Ui, screen: &Screen<'_>, actions: &mut Vec<Action>) {
    let c = theme::colors(ui.ctx());
    let catalog = screen.catalog;
    let line = catalog
        .tr(k::HELP_ABOUT_LINE)
        .arg("version", &screen.environment.app_version)
        .arg("protocol", websign_protocol::version::PROTOCOL_VERSION)
        .to_string();
    ui.horizontal_wrapped(|ui| {
        ui.spacing_mut().item_spacing.x = metrics::SPACE_2;
        ui.label(typography::SMALL.rich(line).color(c.fg_subtle));
        for (key, url) in [
            (k::HELP_SOURCE, websign_project::REPOSITORY.to_owned()),
            (k::HELP_PRIVACY, privacy_page()),
        ] {
            ui.label(typography::SMALL.rich("·").color(c.fg_subtle));
            let link = ui.add(egui::Link::new(
                typography::SMALL.rich(tr(catalog, key)).color(c.accent_fg),
            ));
            if link.clicked() {
                actions.push(Action::OpenUrl(url));
            }
        }
    });
}
