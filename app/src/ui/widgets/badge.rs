//! The certificate type badge (`docs/ux.md` §5.3): neutral on purpose,
//! because color is reserved for state and the type is not a state.

use std::sync::Arc;

use egui::text::Galley;
use egui::{CornerRadius, Pos2, Rect, Response, Sense, Ui, Vec2, Widget, WidgetInfo, WidgetType};

use super::text;
use crate::ui::theme::{self, metrics, typography};

/// Horizontal and vertical padding (`2px 6px` in the mockups).
const PADDING: Vec2 = Vec2::new(6.0, 2.0);

/// A small neutral label ("ICP-Brasil A3", "SHA-256").
#[derive(Debug, Clone, Copy)]
pub struct Badge<'a> {
    label: &'a str,
}

impl<'a> Badge<'a> {
    pub fn new(label: &'a str) -> Self {
        Badge { label }
    }
}

impl<'a> Badge<'a> {
    /// The laid-out label and the badge's full size, for widgets that paint
    /// a badge inside themselves (the certificate row).
    pub fn measure(self, ui: &Ui) -> (Arc<Galley>, Vec2) {
        let muted = theme::colors(ui.ctx()).fg_muted;
        let galley = text::whole(ui, typography::CAPTION.rich(self.label).color(muted));
        let size = galley.size() + 2.0 * PADDING;
        (galley, size)
    }

    /// Paints a measured badge with its top-left corner at `at`.
    pub fn paint(ui: &Ui, at: Pos2, galley: Arc<Galley>, size: Vec2) {
        let c = theme::colors(ui.ctx());
        let rect = Rect::from_min_size(at, size);
        ui.painter()
            .rect_filled(rect, CornerRadius::same(metrics::RADIUS_SM), c.bg_sunken);
        ui.painter().galley(rect.min + PADDING, galley, c.fg_muted);
    }
}

impl Widget for Badge<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let (galley, size) = self.measure(ui);
        let (rect, response) = ui.allocate_exact_size(size, Sense::hover());
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, self.label));
        Badge::paint(ui, rect.min, galley, size);
        response
    }
}
