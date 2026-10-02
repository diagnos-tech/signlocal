//! The pieces every tab is made of: a section label, a list of rows, an
//! empty card and a small note (the mockups' `.d-sec`, `.d-list`, `.note`).

use egui::{Frame, Label, Margin, Stroke, Ui};

use crate::ui::icons::Icon;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::list::{self, Position};
use crate::ui::widgets::text;

/// Space above a section label (`margin: 20px 0 8px`).
const ABOVE: f32 = 20.0;

/// A section label, with an optional 16 px icon.
pub fn label(ui: &mut Ui, text: &str, icon: Option<Icon>) {
    let c = theme::colors(ui.ctx());
    ui.add_space(ABOVE);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = 6.0;
        if let Some(icon) = icon {
            text::icon(ui, icon, metrics::ICON_SM, c.fg_muted);
        }
        ui.label(typography::CAPTION.rich(text).color(c.fg_muted));
    });
    ui.add_space(metrics::SPACE_2);
}

/// `items` as rows of one list; `row` draws item `i` at its position.
pub fn list<T>(ui: &mut Ui, items: &[T], mut row: impl FnMut(&mut Ui, &T, Position)) {
    list::show(ui, |ui| {
        for (index, item) in items.iter().enumerate() {
            row(ui, item, Position::of(index, items.len()));
        }
    });
}

/// A card saying there is nothing to list, and what to do about it.
pub fn empty(ui: &mut Ui, message: &str) {
    let c = theme::colors(ui.ctx());
    Frame::new()
        .fill(c.bg_surface)
        .stroke(Stroke::new(1.0, c.border))
        .corner_radius(metrics::RADIUS_LG)
        .inner_margin(Margin::symmetric(16, 14))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(typography::BODY.rich(message).color(c.fg_muted));
        });
}

/// A small subtle note that wraps.
pub fn note(ui: &mut Ui, message: &str) {
    let c = theme::colors(ui.ctx());
    ui.add(Label::new(typography::SMALL.rich(message).color(c.fg_subtle)).wrap());
}
