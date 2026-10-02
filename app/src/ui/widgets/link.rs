//! A text link painted at a given spot inside another widget (the
//! certificate row's "Details"): `accent-fg` caption text, underlined on
//! hover, an AccessKit `Link` with its own focus ring.

use egui::{Id, Pos2, Rect, Response, Sense, Stroke, Ui, WidgetInfo, WidgetType};

use super::{focus, text};
use crate::ui::theme::{self, metrics, typography};

/// Paints `label` with its top-left corner at `at`; returns its response.
pub fn paint(ui: &mut Ui, id: Id, at: Pos2, label: &str) -> Response {
    let color = theme::colors(ui.ctx()).accent_fg;
    let galley = text::whole(ui, typography::CAPTION.rich(label).color(color));
    let bounds = Rect::from_min_size(at, galley.size());
    let hit = bounds.expand(metrics::SPACE_1);
    let response = ui.interact(hit, id, Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Link, true, label));
    ui.painter().galley(at, galley, color);
    if response.hovered() {
        let y = bounds.max.y - 0.5;
        ui.painter()
            .hline(bounds.x_range(), y, Stroke::new(1.0, color));
    }
    focus::ring(ui, &response, hit, metrics::RADIUS_SM);
    response
}

/// The width `label` takes, to reserve room before painting.
pub fn width(ui: &Ui, label: &str) -> f32 {
    text::whole(ui, typography::CAPTION.rich(label)).size().x
}
