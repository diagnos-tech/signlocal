//! A tab's header (`docs/ux.md` §8.1): title (`text-headline`), subtitle
//! (`fg-muted`) and, on the tabs that scan, [Scan again] on the right
//! (F5 / Ctrl+R).

use egui::{Align, Label, Layout, Ui, Vec2};
use websign_i18n::{Key, k};
use websign_protocol::messages::DiagnosticsTab;

use super::Screen;
use super::sidebar::identity;
use super::words::tr;
use crate::ui::diagnostics::state::Action;
use crate::ui::icons;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::button::Button;
use crate::ui::widgets::spinner;

pub fn show(ui: &mut Ui, screen: &Screen<'_>, tab: DiagnosticsTab, actions: &mut Vec<Action>) {
    let catalog = screen.catalog;
    let title = tr(catalog, identity(tab).1);
    let subtitle = tr(catalog, subtitle(tab));
    let scan = (tab != DiagnosticsTab::Help).then(|| tr(catalog, k::COMMON_REFRESH));
    // [Scan again] first, on the right; title and subtitle wrap in what is
    // left, as the mockups' `.d-head`.
    ui.with_layout(Layout::right_to_left(Align::Min), |ui| {
        ui.spacing_mut().item_spacing.x = metrics::SPACE_4;
        if let Some(scan) = &scan {
            let button = Button::secondary(scan)
                .icon(icons::SCAN_AGAIN)
                .busy(screen.scanning);
            if ui.add(button).clicked() {
                actions.push(Action::Rescan);
            }
        }
        ui.with_layout(Layout::top_down(Align::Min), |ui| {
            titles(ui, &title, &subtitle);
        });
    });
}

fn titles(ui: &mut Ui, title: &str, subtitle: &str) {
    let c = theme::colors(ui.ctx());
    ui.spacing_mut().item_spacing.y = 2.0;
    ui.add(Label::new(typography::HEADLINE.rich(title).color(c.fg)).wrap());
    ui.add(Label::new(typography::BODY.rich(subtitle).color(c.fg_muted)).wrap());
}

fn subtitle(tab: DiagnosticsTab) -> Key {
    match tab {
        DiagnosticsTab::Browsers => k::BROWSERS_SUBTITLE,
        DiagnosticsTab::Devices => k::DEVICES_SUBTITLE,
        DiagnosticsTab::Certificates => k::CERTS_TAB_SUBTITLE,
        DiagnosticsTab::Help => k::HELP_SUBTITLE,
    }
}

/// While the first scan runs: a spinner and "Please wait…".
pub fn loading(ui: &mut Ui, screen: &Screen<'_>) {
    let c = theme::colors(ui.ctx());
    ui.add_space(metrics::SPACE_5);
    ui.horizontal(|ui| {
        let (rect, _) = ui.allocate_exact_size(Vec2::splat(metrics::ICON_SM), egui::Sense::hover());
        spinner::paint(ui, rect, c.fg_muted);
        ui.label(
            typography::BODY
                .rich(tr(screen.catalog, k::COMMON_WAIT))
                .color(c.fg_muted),
        );
    });
}
