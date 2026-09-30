//! Loading placeholders (§4.4 code skeleton, §4.8 two-row list skeleton).
//!
//! Static on purpose: a shimmer would be one more animation to switch off
//! for reduced motion, and the text next to it ("Preparing the document…")
//! already says that something is on its way. Hidden from screen readers.

use egui::{CornerRadius, Response, Sense, Ui, Vec2, Widget};

use crate::ui::theme::{self, metrics};

/// A rounded placeholder block of a given size.
#[derive(Debug, Clone, Copy)]
pub struct Skeleton {
    size: Vec2,
    radius: u8,
}

impl Skeleton {
    pub fn new(size: Vec2) -> Self {
        Skeleton {
            size,
            radius: metrics::RADIUS_SM,
        }
    }

    /// A text line placeholder of `width` for a token of `line_height`.
    pub fn line(width: f32, line_height: f32) -> Self {
        // Two thirds of the line: the glyphs' visual height, not the leading.
        Skeleton::new(Vec2::new(width, (line_height * 2.0 / 3.0).round()))
    }

    pub fn size(&self) -> Vec2 {
        self.size
    }

    pub fn radius(mut self, radius: u8) -> Self {
        self.radius = radius;
        self
    }
}

impl Widget for Skeleton {
    fn ui(self, ui: &mut Ui) -> Response {
        let (rect, response) = ui.allocate_exact_size(self.size, Sense::hover());
        let fill = theme::colors(ui.ctx()).bg_sunken;
        ui.painter()
            .rect_filled(rect, CornerRadius::same(self.radius), fill);
        response
    }
}
