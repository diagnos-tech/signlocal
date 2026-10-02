//! Laying out one run of text for widgets that paint by hand, and painting
//! decorative icons.

use std::sync::Arc;

use egui::text::Galley;
use egui::{Color32, Rect, RichText, Sense, TextStyle, TextWrapMode, Ui, Vec2, WidgetText};

use crate::ui::icons::Icon;

/// `text` on one line, cut with "…" when wider than `max_width` (§5.1:
/// names and issuers truncate at the end; the full text goes in the tooltip
/// and the accessible name).
pub fn line(ui: &Ui, text: RichText, max_width: f32) -> Arc<Galley> {
    WidgetText::from(text).into_galley(
        ui,
        Some(TextWrapMode::Truncate),
        max_width.max(0.0),
        TextStyle::Body,
    )
}

/// `text` on one line at its natural width, never cut.
pub fn whole(ui: &Ui, text: RichText) -> Arc<Galley> {
    WidgetText::from(text).into_galley(
        ui,
        Some(TextWrapMode::Extend),
        f32::INFINITY,
        TextStyle::Body,
    )
}

/// `text` wrapped to `width`.
pub fn wrapped(ui: &Ui, text: RichText, width: f32) -> Arc<Galley> {
    WidgetText::from(text).into_galley(
        ui,
        Some(TextWrapMode::Wrap),
        width.max(1.0),
        TextStyle::Body,
    )
}

/// A decorative icon of `size`, painted in a `size` square laid out like a
/// widget but with no AccessKit node: a label would hand screen readers
/// the glyph, which they read as "private use character".
pub fn icon(ui: &mut Ui, icon: Icon, size: f32, color: Color32) -> Rect {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(size), Sense::hover());
    let glyph = whole(ui, icon.rich(size, color));
    let at = rect.center() - glyph.size() / 2.0;
    ui.painter().galley(at, glyph, color);
    rect
}
