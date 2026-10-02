//! The "Remember" checkbox (`docs/ux.md` §4.10): a 16 px box with a
//! `border-strong` outline (≥ 3:1), filled with the accent and a check mark
//! when checked; label and help text beside it. The whole block is the
//! target and one accessible checkbox named by its label.

use egui::{
    CornerRadius, Response, Sense, Stroke, StrokeKind, Ui, Vec2, WidgetInfo, WidgetType, pos2,
};

use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::{focus, text};

const BOX: f32 = 16.0;
const GAP: f32 = 10.0;
const DISABLED_OPACITY: f32 = 0.42;

/// Draws the checkbox with `label` and `help`; `clicked()` toggles it.
pub fn show(ui: &mut Ui, checked: bool, enabled: bool, label: &str, help: &str) -> Response {
    let c = theme::colors(ui.ctx());
    let width = ui.available_width() - BOX - GAP;
    let title = text::wrapped(ui, typography::BUTTON.rich(label).color(c.fg), width);
    let help = text::wrapped(ui, typography::SMALL.rich(help).color(c.fg_subtle), width);
    let height = (title.size().y + help.size().y).max(metrics::CONTROL_SM);
    let sense = if enabled {
        Sense::click()
    } else {
        Sense::focusable_noninteractive()
    };
    let (rect, response) = ui.allocate_exact_size(Vec2::new(ui.available_width(), height), sense);
    response.widget_info(|| WidgetInfo::selected(WidgetType::Checkbox, enabled, checked, label));
    let opacity = if enabled { 1.0 } else { DISABLED_OPACITY };
    let painter = ui.painter();
    let square = egui::Rect::from_min_size(pos2(rect.min.x, rect.min.y + 2.0), Vec2::splat(BOX));
    let (fill, border) = if checked {
        (c.accent, c.accent)
    } else {
        (c.bg_surface, c.border_strong)
    };
    painter.rect(
        square,
        CornerRadius::same(metrics::RADIUS_SM),
        fill.gamma_multiply(opacity),
        Stroke::new(1.5, border.gamma_multiply(opacity)),
        StrokeKind::Inside,
    );
    if checked {
        let mark = [
            pos2(square.min.x + 4.0, square.center().y),
            pos2(square.min.x + 7.0, square.max.y - 4.5),
            pos2(square.max.x - 3.5, square.min.y + 4.5),
        ];
        painter.line(
            mark.to_vec(),
            Stroke::new(2.0, c.on_accent.gamma_multiply(opacity)),
        );
    }
    let x = rect.min.x + BOX + GAP;
    let title_height = title.size().y;
    painter.galley(pos2(x, rect.min.y), title, c.fg.gamma_multiply(opacity));
    painter.galley(pos2(x, rect.min.y + title_height), help, c.fg_subtle);
    focus::ring(ui, &response, square, metrics::RADIUS_SM);
    response
}
