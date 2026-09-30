//! Status chips (`docs/ux.md` §4.3, §8.1): a 22 px pill with an icon and a
//! short text, e.g. "Allowed site", "New site", "Ready to sign".

use egui::{
    CornerRadius, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, WidgetInfo, WidgetType,
};

use super::text;
use super::tone::Tone;
use crate::ui::icons::Icon;
use crate::ui::theme::{self, metrics, typography};

/// Left padding, right padding and icon-to-text gap (`0 9px 0 7px`, gap 6).
const LEFT: f32 = 7.0;
const RIGHT: f32 = 9.0;
const GAP: f32 = 6.0;
const ICON: f32 = 14.0;

/// A tone, an icon and a text; the icon defaults to the tone's.
#[derive(Debug, Clone, Copy)]
pub struct Chip<'a> {
    tone: Tone,
    icon: Icon,
    label: &'a str,
    accessible: Option<&'a str>,
}

impl<'a> Chip<'a> {
    pub fn new(tone: Tone, label: &'a str) -> Self {
        Chip {
            tone,
            icon: tone.icon(),
            label,
            accessible: None,
        }
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    /// A longer name for screen readers ("New site" is read as "First time
    /// this site asks for anything on this computer", §4.3).
    pub fn accessible_name(mut self, name: &'a str) -> Self {
        self.accessible = Some(name);
        self
    }
}

impl Widget for Chip<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let palette = self.tone.palette(&theme::colors(ui.ctx()));
        let label = text::whole(ui, typography::CAPTION.rich(self.label).color(palette.text));
        let glyph = text::whole(ui, self.icon.rich(ICON, palette.icon));
        let width = LEFT + ICON + GAP + label.size().x + RIGHT;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(width, metrics::CHIP_HEIGHT), Sense::hover());
        let name = self.accessible.unwrap_or(self.label);
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, name));
        let painter = ui.painter();
        let radius = CornerRadius::same((metrics::CHIP_HEIGHT / 2.0) as u8);
        painter.rect(
            rect,
            radius,
            palette.fill,
            Stroke::new(1.0, palette.border),
            StrokeKind::Inside,
        );
        let middle = rect.center().y;
        let icon_x = rect.min.x + LEFT + (ICON - glyph.size().x) / 2.0;
        painter.galley(
            egui::pos2(icon_x, middle - glyph.size().y / 2.0),
            glyph,
            palette.icon,
        );
        let text_pos = egui::pos2(
            rect.min.x + LEFT + ICON + GAP,
            middle - label.size().y / 2.0,
        );
        painter.galley(text_pos, label, palette.text);
        response
    }
}
