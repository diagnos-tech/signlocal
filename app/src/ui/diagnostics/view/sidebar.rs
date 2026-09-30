//! The sidebar (`docs/ux.md` §8.1): brand and version, the overall light,
//! the four tabs as an AccessKit tab list, and "Copy diagnostics" at the
//! bottom with its confirmation.

use egui::accesskit::Role;
use egui::{Align, CornerRadius, Layout, Sense, Ui, UiBuilder, Vec2, vec2};
use websign_i18n::k;
use websign_protocol::messages::DiagnosticsTab;

use super::Screen;
use super::words::tr;
use crate::ui::diagnostics::lights;
use crate::ui::diagnostics::state::{Action, Notice, TABS};
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::banner::Banner;
use crate::ui::widgets::button::Button;
use crate::ui::widgets::chip::Chip;
use crate::ui::widgets::tabs::{Tab, TabStatus};
use crate::ui::widgets::text;
use crate::ui::widgets::tone::Tone;

/// Brand mark size (`24 px accent square`).
const MARK: f32 = 24.0;

pub fn show(ui: &mut Ui, screen: &Screen<'_>, actions: &mut Vec<Action>) {
    let inner = ui.max_rect().shrink2(vec2(12.0, 16.0));
    let mut ui = ui.new_child(UiBuilder::new().max_rect(inner));
    ui.spacing_mut().item_spacing = vec2(0.0, 14.0);
    brand(&mut ui, screen);
    if let Some(lights) = screen.lights {
        let (icon, tone, key) = lights::look(lights.overall());
        let label = tr(screen.catalog, key);
        ui.horizontal(|ui| {
            ui.add_space(metrics::SPACE_1);
            ui.add(Chip::new(tone, &label).icon(icon));
        });
    }
    tabs(&mut ui, screen, actions);
    footer(&mut ui, screen, actions);
}

fn brand(ui: &mut Ui, screen: &Screen<'_>) {
    let c = theme::colors(ui.ctx());
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 10.0;
        ui.add_space(metrics::SPACE_1);
        let (mark, _) = ui.allocate_exact_size(Vec2::splat(MARK), Sense::hover());
        ui.painter()
            .rect_filled(mark, CornerRadius::same(metrics::RADIUS_MD), c.accent);
        let glyph = text::whole(ui, icons::BRAND.rich(metrics::ICON_SM, c.on_accent));
        ui.painter()
            .galley(mark.center() - glyph.size() / 2.0, glyph, c.on_accent);
        ui.vertical(|ui| {
            ui.spacing_mut().item_spacing.y = 0.0;
            ui.label(
                typography::TITLE
                    .rich(websign_project::PRODUCT_NAME)
                    .color(c.fg),
            );
            ui.label(
                typography::SMALL
                    .rich(&screen.environment.app_version)
                    .color(c.fg_subtle),
            );
        });
    });
}

fn tabs(ui: &mut Ui, screen: &Screen<'_>, actions: &mut Vec<Action>) {
    let list = ui.scope_builder(UiBuilder::new().id_salt("tablist"), |ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        let responses: Vec<egui::Response> = TABS
            .into_iter()
            .map(|tab| {
                let response = tab_button(ui, screen, tab);
                if response.clicked() {
                    actions.push(Action::SelectTab(tab));
                }
                response
            })
            .collect();
        arrows(ui, &responses, actions);
        ui.unique_id()
    });
    let name = tr(screen.catalog, k::DIAG_WINDOW_TITLE);
    let rect = list.response.rect;
    ui.ctx().accesskit_node_builder(list.inner, |node| {
        node.set_role(Role::TabList);
        node.set_label(name);
        // Where the list is, so a screen reader can show it.
        node.set_bounds(egui::accesskit::Rect {
            x0: rect.min.x.into(),
            y0: rect.min.y.into(),
            x1: rect.max.x.into(),
            y1: rect.max.y.into(),
        });
    });
}

fn tab_button(ui: &mut Ui, screen: &Screen<'_>, tab: DiagnosticsTab) -> egui::Response {
    let (icon, key) = identity(tab);
    let label = tr(screen.catalog, key);
    let status = screen
        .lights
        .and_then(|lights| lights.of_tab(tab))
        .map(|light| {
            let (icon, tone, key) = lights::look(light);
            (icon, tone, tr(screen.catalog, key))
        });
    Tab {
        icon,
        label: &label,
        status: status.as_ref().map(|(icon, tone, name)| TabStatus {
            icon: *icon,
            tone: *tone,
            name,
        }),
        selected: screen.tab() == tab,
    }
    .show(ui)
}

/// ↑/↓ on a focused tab selects and focuses its neighbor, wrapping around
/// (`docs/ux.md` §8.8, the WAI-ARIA tabs pattern).
fn arrows(ui: &Ui, responses: &[egui::Response], actions: &mut Vec<Action>) {
    let Some(focused) = responses.iter().position(egui::Response::has_focus) else {
        return;
    };
    let step = ui.input_mut(|input| {
        if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
            Some(1)
        } else if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
            Some(TABS.len() - 1)
        } else {
            None
        }
    });
    if let Some(step) = step {
        let next = (focused + step) % TABS.len();
        responses[next].request_focus();
        // egui would also move focus down or up from the new tab this frame.
        ui.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
        actions.push(Action::SelectTab(TABS[next]));
    }
}

fn footer(ui: &mut Ui, screen: &Screen<'_>, actions: &mut Vec<Action>) {
    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.y = metrics::SPACE_2;
        let label = tr(screen.catalog, k::DIAG_COPY);
        let copy = Button::secondary(&label)
            .icon(icons::COPY)
            .enabled(screen.report.is_some());
        if ui.add(copy).clicked() {
            actions.push(Action::CopyReport);
        }
        if screen.state.notice_at(screen.clock) == Some(&Notice::Copied) {
            let message = tr(screen.catalog, k::DIAG_COPIED);
            ui.add(Banner::new(Tone::Success, &message).compact());
        }
    });
}

/// A tab's icon and name.
pub fn identity(tab: DiagnosticsTab) -> (Icon, websign_i18n::Key) {
    match tab {
        DiagnosticsTab::Browsers => (icons::BROWSERS, k::DIAG_TAB_BROWSERS),
        DiagnosticsTab::Devices => (icons::TOKEN, k::DIAG_TAB_DEVICES),
        DiagnosticsTab::Certificates => (icons::CERTIFICATE, k::DIAG_TAB_CERTS),
        DiagnosticsTab::Help => (icons::HELP, k::DIAG_TAB_HELP),
    }
}
