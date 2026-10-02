//! The list container (`docs/ux.md` §4.2, §8.3): `bg-surface`, a 1 px
//! `border` outline, `radius-lg`, rows separated by 1 px dividers.
//!
//! Rows paint their own background (selected, hover), so they need to know
//! where they sit: the first and last round their outer corners to follow
//! the container, which is what CSS `overflow: hidden` does in the mockups.
//! Without it a selected first row paints square corners over the outline.

use egui::{CornerRadius, Frame, InnerResponse, Margin, Stroke, Ui, Vec2};

use crate::ui::theme::{self, metrics};

/// Where a row sits in its list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Position {
    pub first: bool,
    pub last: bool,
}

impl Position {
    /// Row `index` of a list of `len` rows.
    pub fn of(index: usize, len: usize) -> Self {
        Position {
            first: index == 0,
            last: index + 1 >= len,
        }
    }

    /// Every row but the first draws the divider on its top edge.
    pub fn divider(self) -> bool {
        !self.first
    }

    /// The row background's corners: the container's radius minus its
    /// outline, on the outer corners only.
    pub fn corners(self) -> CornerRadius {
        let outer = metrics::RADIUS_LG - 1;
        let (top, bottom) = (
            if self.first { outer } else { 0 },
            if self.last { outer } else { 0 },
        );
        CornerRadius {
            nw: top,
            ne: top,
            sw: bottom,
            se: bottom,
        }
    }
}

/// Draws the container and lays `rows` out inside it with no spacing; rows
/// start inside the 1 px outline, so their fills never cover it.
pub fn show<R>(ui: &mut Ui, rows: impl FnOnce(&mut Ui) -> R) -> InnerResponse<R> {
    let c = theme::colors(ui.ctx());
    Frame::new()
        .fill(c.bg_surface)
        .stroke(Stroke::new(1.0, c.border))
        .corner_radius(metrics::RADIUS_LG)
        .inner_margin(Margin::ZERO)
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = Vec2::ZERO;
            rows(ui)
        })
}
