//! The 16 px selection circle of list rows (`docs/ux.md` §5.1): an outline
//! in `border-strong` when off, a filled `accent` disc with an `on-accent`
//! dot when on. Painted inside the row, which owns the click and the
//! accessible radio semantics.

use egui::{Painter, Pos2, Stroke};

use crate::ui::theme::Colors;

pub const SIZE: f32 = 16.0;
const DOT: f32 = 3.0;

/// Paints the circle centered on `center`.
pub fn paint(painter: &Painter, center: Pos2, selected: bool, c: &Colors) {
    let radius = SIZE / 2.0;
    if selected {
        painter.circle_filled(center, radius, c.accent);
        painter.circle_filled(center, DOT, c.on_accent);
    } else {
        painter.circle(
            center,
            radius - 0.75,
            c.bg_surface,
            Stroke::new(1.5, c.border_strong),
        );
    }
}
