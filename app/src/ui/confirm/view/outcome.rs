//! How a request ended (`docs/ux.md` §4.8, §4.10, §4.11): "Signed", the
//! certificate sent, the site that gave up, or the expiry. The result was
//! already sent; the window closes by itself after the hold. Each is a
//! polite live region so screen readers say how it ended.

use egui::{Align, Layout, Ui, Vec2, WidgetInfo, WidgetType};
use websign_i18n::k;
use websign_ui_model::confirm::ConfirmState;
use websign_ui_model::confirm::port::Mode;
use websign_ui_model::confirm::view::Expiry;

use super::Screen;
use crate::ui::confirm::words;
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, metrics, typography};

const ICON: f32 = 48.0;
/// Room above the notice, so it sits in the upper middle of the body.
const TOP: f32 = 64.0;

pub fn show(ui: &mut Ui, s: &mut Screen<'_>) {
    let c = theme::colors(ui.ctx());
    let tr = s.tr;
    let site = words::site(&s.view.header.caller);
    let (icon, color, title, body) = match (&s.view.state, s.view.mode) {
        (ConfirmState::Success, Mode::Sign { .. }) => (
            icons::SUCCESS,
            c.success,
            tr.tr(k::STATE_SUCCESS_TITLE).to_string(),
            Some(tr.tr(k::STATE_SUCCESS_BODY).arg("site", &site).to_string()),
        ),
        (ConfirmState::Success, Mode::Choose) => (
            icons::SUCCESS,
            c.success,
            tr.tr(k::STATE_SELECT_SUCCESS)
                .arg("site", &site)
                .to_string(),
            None,
        ),
        (ConfirmState::SiteCancelled, _) => (
            icons::INFO,
            c.fg_subtle,
            tr.tr(k::STATE_SITE_CANCELLED)
                .arg("site", &site)
                .to_string(),
            None,
        ),
        // The site's `prepare` hung, not the person: say so.
        _ if s.view.expiry == Some(Expiry::Digest) => (
            icons::EXPIRES_SOON,
            c.warning,
            tr.tr(k::ERRORS_DIGEST_TIMEOUT_TITLE).to_string(),
            Some(
                tr.tr(k::ERRORS_DIGEST_TIMEOUT_BODY)
                    .arg("site", &site)
                    .to_string(),
            ),
        ),
        _ => (
            icons::EXPIRES_SOON,
            c.warning,
            tr.tr(k::ERRORS_TIMEOUT_TITLE).to_string(),
            Some(tr.tr(k::ERRORS_TIMEOUT_BODY).to_string()),
        ),
    };
    notice(ui, icon, color, &title, body.as_deref());
}

fn notice(ui: &mut Ui, icon: Icon, color: egui::Color32, title: &str, body: Option<&str>) {
    let c = theme::colors(ui.ctx());
    ui.add_space(TOP);
    let response = ui
        .with_layout(Layout::top_down(Align::Center), |ui| {
            ui.spacing_mut().item_spacing = Vec2::new(0.0, metrics::SPACE_1);
            ui.add_sized(Vec2::splat(ICON), egui::Label::new(icon.rich(ICON, color)));
            ui.add_space(6.0);
            ui.label(typography::TITLE.rich(title).color(c.fg));
            if let Some(body) = body {
                ui.scope(|ui| {
                    ui.set_max_width(320.0);
                    ui.add(egui::Label::new(typography::BODY.rich(body).color(c.fg_muted)).wrap());
                });
            }
        })
        .response;
    let spoken = match body {
        Some(body) => format!("{title}. {body}"),
        None => title.to_owned(),
    };
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &spoken));
    ui.ctx().accesskit_node_builder(response.id, |node| {
        node.set_role(egui::accesskit::Role::Status);
        node.set_live(egui::accesskit::Live::Polite);
    });
}
