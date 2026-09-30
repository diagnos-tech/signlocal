//! A one-line text input in the shared field frame, with an optional
//! leading icon: the certificate filter (§5.13, `magnifying-glass`).
//!
//! For a PIN use [`super::pin_field::PinField`]: egui's `TextEdit` keeps
//! copies of what is typed (undo history), which a secret must not leave.

use egui::{Rect, Response, TextEdit, Ui, Vec2, pos2, vec2};

use super::{field, text};
use crate::ui::icons::Icon;
use crate::ui::theme::{self, metrics, typography};

/// Left padding (10) and the icon-to-text gap.
const PADDING: f32 = 10.0;

/// An input over `text`.
#[derive(Debug)]
pub struct TextField<'a> {
    text: &'a mut String,
    /// Accessible name (the visible label, or the placeholder when there is
    /// no label).
    name: &'a str,
    hint: &'a str,
    icon: Option<Icon>,
    error: bool,
}

impl<'a> TextField<'a> {
    pub fn new(text: &'a mut String, name: &'a str) -> Self {
        TextField {
            text,
            name,
            hint: "",
            icon: None,
            error: false,
        }
    }

    /// Placeholder shown while empty, in `fg-subtle`.
    pub fn hint(mut self, hint: &'a str) -> Self {
        self.hint = hint;
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = Some(icon);
        self
    }

    pub fn error(mut self, error: bool) -> Self {
        self.error = error;
        self
    }

    pub fn show(self, ui: &mut Ui) -> Response {
        let c = theme::colors(ui.ctx());
        let size = vec2(ui.available_width(), metrics::CONTROL_MD);
        let (rect, _) = ui.allocate_exact_size(size, egui::Sense::hover());
        field::paint(ui, rect, self.error);
        let mut left = rect.min.x + PADDING;
        if let Some(icon) = self.icon {
            let glyph = text::whole(ui, icon.rich(metrics::ICON_SM, c.fg_subtle));
            let y = rect.center().y - glyph.size().y / 2.0;
            ui.painter().galley(pos2(left, y), glyph, c.fg_subtle);
            left += metrics::ICON_SM + metrics::SPACE_2;
        }
        let height = typography::BODY.line_height;
        let inner = Rect::from_min_size(
            pos2(left, rect.center().y - height / 2.0),
            Vec2::new(rect.max.x - PADDING - left, height),
        );
        let edit = TextEdit::singleline(self.text)
            .frame(egui::Frame::NONE)
            .font(typography::BODY.font_id())
            .text_color(c.fg)
            .hint_text(typography::BODY.rich(self.hint).color(c.fg_subtle))
            .desired_width(inner.width());
        let response = ui.put(inner, edit);
        let name = self.name;
        ui.ctx()
            .accesskit_node_builder(response.id, |node| node.set_label(name));
        field::ring(ui, &response, rect);
        response
    }
}
