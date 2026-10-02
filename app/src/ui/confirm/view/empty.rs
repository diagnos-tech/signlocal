//! Before there is a list (`docs/ux.md` §4.8, §6.2): "Looking for
//! certificates…" with a two-row skeleton after 150 ms and the slow-driver
//! hint after 2 s; and the empty state with the devices that brought no
//! certificate.

use egui::{Align, Layout, Sense, Ui, Vec2};
use websign_i18n::k;
use websign_ui_model::confirm::port::Mode;
use websign_ui_model::confirm::view::LoadingView;

use super::{Screen, possible};
use crate::ui::icons;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::list::{self};
use crate::ui::widgets::skeleton::Skeleton;
use crate::ui::widgets::spinner::Spinner;
use crate::ui::widgets::text;

const EMPTY_ICON: f32 = 28.0;
const EMPTY_TEXT_WIDTH: f32 = 300.0;

pub fn loading(ui: &mut Ui, s: &mut Screen<'_>) {
    let c = theme::colors(ui.ctx());
    label(ui, s);
    let loading = s.view.loading.clone().unwrap_or(LoadingView {
        skeleton: false,
        slow_device: None,
    });
    let status = match &loading.slow_device {
        Some(device) => {
            s.tr.tr(k::CERTS_LOADING_SLOW)
                .arg("device", device)
                .to_string()
        }
        None => s.tr.tr(k::CERTS_LOADING).to_string(),
    };
    list::show(ui, |ui| {
        ui.set_width(ui.available_width());
        if loading.skeleton {
            for index in 0..3 {
                skeleton_row(ui, index > 0, index < 2);
            }
        }
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(16, 12))
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(metrics::SPACE_2, 0.0);
                    ui.add(Spinner::new(&status).size(metrics::ICON_SM));
                    let width = ui.available_width();
                    let rich = typography::SMALL.rich(&status).color(c.fg_muted);
                    ui.add(egui::Label::new(text::wrapped(ui, rich, width)));
                });
            });
    });
}

/// A placeholder in the shape of a certificate row: the radio and the
/// three text lines, painted like `widgets::skeleton` (static, no shimmer).
/// Without `bars` it is only the divider above the status line.
fn skeleton_row(ui: &mut Ui, divider: bool, bars: bool) {
    let c = theme::colors(ui.ctx());
    let height = if bars { metrics::ROW_CERT } else { 1.0 };
    let (rect, _) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());
    let painter = ui.painter();
    if divider {
        painter.hline(
            rect.x_range(),
            rect.min.y + 0.5,
            egui::Stroke::new(1.0, c.border),
        );
    }
    if !bars {
        return;
    }
    let radio = egui::pos2(rect.min.x + 16.0 + 8.0, rect.min.y + 20.0);
    painter.circle_filled(radio, 8.0, c.bg_sunken);
    let left = rect.min.x + 16.0 + 16.0 + 12.0;
    // (top of the line, width, line height): name, detail, location lines.
    for (top, width, line) in [
        (10.0, 180.0, 20.0),
        (30.0, 240.0, 16.0),
        (46.0, 150.0, 16.0),
    ] {
        let bar = Skeleton::line(width, line).size();
        let at = egui::pos2(left, rect.min.y + top + (line - bar.y) / 2.0);
        painter.rect_filled(
            egui::Rect::from_min_size(at, bar),
            metrics::RADIUS_SM,
            c.bg_sunken,
        );
    }
}

pub fn show(ui: &mut Ui, s: &mut Screen<'_>) {
    let c = theme::colors(ui.ctx());
    label(ui, s);
    let title = s.tr.tr(k::CERTS_EMPTY_TITLE).to_string();
    let body = s.tr.tr(k::CERTS_EMPTY_BODY).to_string();
    let possible = s.view.possible.clone();
    list::show(ui, |ui| {
        ui.set_width(ui.available_width());
        egui::Frame::new()
            .inner_margin(egui::Margin::symmetric(16, 20))
            .show(ui, |ui| {
                ui.with_layout(Layout::top_down(Align::Center), |ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(0.0, metrics::SPACE_1);
                    text::icon(ui, icons::CERTIFICATE, EMPTY_ICON, c.fg_subtle);
                    ui.label(typography::BODY_STRONG.rich(&title).color(c.fg));
                    ui.scope(|ui| {
                        ui.set_max_width(EMPTY_TEXT_WIDTH.min(ui.available_width()));
                        ui.add(
                            egui::Label::new(typography::SMALL.rich(&body).color(c.fg_muted))
                                .wrap(),
                        );
                    });
                });
            });
        for card in &possible {
            let rect = ui.available_rect_before_wrap();
            ui.painter().hline(
                rect.x_range(),
                rect.min.y + 0.5,
                egui::Stroke::new(1.0, c.border),
            );
            possible::show_card(ui, s, card);
        }
    });
}

fn label(ui: &mut Ui, s: &Screen<'_>) {
    let c = theme::colors(ui.ctx());
    let label = match s.view.mode {
        Mode::Sign { .. } => s.tr.tr(k::CERTS_LABEL_SIGN),
        Mode::Choose => s.tr.tr(k::CERTS_LABEL_SELECT),
    };
    ui.label(
        typography::CAPTION
            .rich(label.to_string())
            .color(c.fg_muted),
    );
    ui.add_space(metrics::SPACE_2);
}
