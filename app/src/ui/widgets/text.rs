//! Laying out one run of text for widgets that paint by hand.

use std::sync::Arc;

use egui::text::Galley;
use egui::{RichText, TextStyle, TextWrapMode, Ui, WidgetText};

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
