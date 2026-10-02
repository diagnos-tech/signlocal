//! The frame every input shares (`docs/ux.md` §4.6, §5.13): `control-md`
//! height, `bg-sunken`, 1 px `border-strong` (`danger` on error),
//! `radius-md`, and the focus ring while the input has focus.

use egui::{CornerRadius, Rect, Response, Stroke, StrokeKind, Ui};

use super::focus;
use crate::ui::theme::{self, metrics};

/// Paints the frame behind an input occupying `rect`.
pub fn paint(ui: &Ui, rect: Rect, error: bool) {
    let c = theme::colors(ui.ctx());
    let border = if error { c.danger } else { c.border_strong };
    ui.painter().rect(
        rect,
        CornerRadius::same(metrics::RADIUS_MD),
        c.bg_sunken,
        Stroke::new(1.0, border),
        StrokeKind::Inside,
    );
}

/// The focus ring around the frame, shown while `input` has focus.
pub fn ring(ui: &Ui, input: &Response, rect: Rect) {
    focus::ring(ui, input, rect, metrics::RADIUS_MD);
}
