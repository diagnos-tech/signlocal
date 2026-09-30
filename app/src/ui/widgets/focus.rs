//! The focus ring (`docs/ux.md` §14): 2 px in the `focus` color, 2 px away
//! from the element, on every focusable widget, never removed. egui's own
//! focus look is a background change, too weak for keyboard users.

use egui::{CornerRadius, Rect, Response, Stroke, StrokeKind, Ui};

use crate::ui::theme::{self, metrics};

/// Draws the ring around `rect` when `response` has keyboard focus: a 2 px
/// gap, then 2 px of ring, like CSS `outline-offset: 2px` in the mockups.
pub fn ring(ui: &Ui, response: &Response, rect: Rect, radius: u8) {
    if response.has_focus() {
        let grow = metrics::FOCUS_OFFSET + metrics::FOCUS_WIDTH / 2.0;
        let radius = CornerRadius::same(radius.saturating_add(grow as u8));
        paint(ui, rect.expand(grow), radius);
    }
}

/// Draws the ring just inside `rect`, for elements that fill their parent
/// edge to edge (list rows, sidebar tabs), where an outside ring would be
/// clipped or cover the neighbor.
pub fn ring_inside(ui: &Ui, response: &Response, rect: Rect, radius: CornerRadius) {
    if response.has_focus() {
        paint(ui, rect.shrink(metrics::FOCUS_WIDTH / 2.0), radius);
    }
}

fn paint(ui: &Ui, rect: Rect, radius: CornerRadius) {
    let color = theme::colors(ui.ctx()).focus;
    ui.painter().rect_stroke(
        rect,
        radius,
        Stroke::new(metrics::FOCUS_WIDTH, color),
        StrokeKind::Middle,
    );
}
