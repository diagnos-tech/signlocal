//! A diagnostics sidebar tab (`docs/ux.md` §8.1): 36 px, icon + label +
//! a status icon on the right; selected in `accent-soft` with `accent-fg`
//! text. Exposed as an AccessKit `Tab`; the window wraps the tabs in a
//! `TabList` and moves between them with ↑/↓ and Ctrl/⌘+1…4 (§8.8).

use egui::accesskit::Role;
use egui::{CornerRadius, Response, Sense, Ui, WidgetInfo, WidgetType, pos2, vec2};

use super::tone::Tone;
use super::{focus, text};
use crate::ui::icons::Icon;
use crate::ui::theme::{self, metrics, typography};

const HEIGHT: f32 = 36.0;
const PADDING: f32 = 10.0;

/// A tab's traffic light: icon shape + color + a name for screen readers,
/// never color alone (§8.1).
#[derive(Debug, Clone, Copy)]
pub struct TabStatus<'a> {
    pub icon: Icon,
    pub tone: Tone,
    /// "Needs attention", read after the tab name.
    pub name: &'a str,
}

/// One tab.
#[derive(Debug, Clone, Copy)]
pub struct Tab<'a> {
    pub icon: Icon,
    pub label: &'a str,
    pub status: Option<TabStatus<'a>>,
    pub selected: bool,
}

impl Tab<'_> {
    pub fn show(self, ui: &mut Ui) -> Response {
        let c = theme::colors(ui.ctx());
        let (rect, response) =
            ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::click());
        let name = match self.status {
            Some(status) => format!("{}, {}", self.label, status.name),
            None => self.label.to_owned(),
        };
        response
            .widget_info(|| WidgetInfo::selected(WidgetType::Button, true, self.selected, &name));
        let selected = self.selected;
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.set_role(Role::Tab);
            // Tabs are selected, not toggled like the checkbox egui assumes.
            node.clear_toggled();
            node.set_selected(selected);
        });

        let fill = if self.selected {
            c.accent_soft
        } else if response.hovered() {
            c.bg_hover
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.painter()
            .rect_filled(rect, CornerRadius::same(metrics::RADIUS_MD), fill);
        let ink = if self.selected { c.accent_fg } else { c.fg };
        let middle = rect.center().y;
        let glyph = text::whole(ui, self.icon.rich(metrics::ICON_MD, ink));
        let icon_pos = pos2(rect.min.x + PADDING, middle - glyph.size().y / 2.0);
        ui.painter().galley(icon_pos, glyph, ink);

        let mut right = rect.max.x - PADDING;
        if let Some(status) = self.status {
            let color = status.tone.palette(&c).icon;
            let glyph = text::whole(ui, status.icon.rich(metrics::ICON_SM, color));
            right -= glyph.size().x;
            ui.painter()
                .galley(pos2(right, middle - glyph.size().y / 2.0), glyph, color);
            right -= metrics::SPACE_2;
        }
        let left = rect.min.x + PADDING + metrics::ICON_MD + PADDING;
        let label = text::line(
            ui,
            typography::BUTTON.rich(self.label).color(ink),
            right - left,
        );
        ui.painter()
            .galley(pos2(left, middle - label.size().y / 2.0), label, ink);
        focus::ring_inside(ui, &response, rect, CornerRadius::same(metrics::RADIUS_MD));
        response
    }
}
