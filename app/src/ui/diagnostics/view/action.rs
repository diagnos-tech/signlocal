//! A row's trailing action: a small secondary button ("Repair", "Revoke",
//! "Download for Windows") or a 28 px icon button ("Copy" next to an ATR).

use egui::{CornerRadius, Rect, Sense, Ui, UiBuilder, Vec2, WidgetInfo, WidgetType};

use crate::ui::icons::Icon;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::button::{Button, Size};
use crate::ui::widgets::{focus, text};

/// Padding of a small secondary button (`.btn.sm`), as the widget draws it.
const BUTTON_PADDING: f32 = 12.0;

/// The action.
#[derive(Debug, Clone, Copy)]
pub enum Trailing<'a> {
    Button {
        label: &'a str,
        icon: Option<Icon>,
    },
    /// `name` is the accessible name and tooltip.
    Icon {
        icon: Icon,
        name: &'a str,
    },
}

/// Whether the action was used this frame.
#[derive(Debug, Default)]
pub struct TrailingResponse {
    pub clicked: bool,
}

impl Trailing<'_> {
    /// The width the action takes, to leave the rest to the row's text.
    pub fn width(&self, ui: &Ui) -> f32 {
        match self {
            Trailing::Button { label, icon } => {
                let label = text::whole(ui, typography::BUTTON.rich(*label)).size().x;
                let lead = icon.map_or(0.0, |_| metrics::ICON_SM + metrics::SPACE_2);
                (label + lead + 2.0 * BUTTON_PADDING).max(metrics::CONTROL_MD)
            }
            Trailing::Icon { .. } => metrics::CONTROL_SM,
        }
    }

    /// Draws the action vertically centered in `rect`.
    /// `id` keeps the action's identity (and keyboard focus) when its label
    /// changes, as "Revoke" does into "Confirm revoke".
    pub fn show(self, ui: &mut Ui, rect: Rect, id: egui::Id) -> TrailingResponse {
        let layout = egui::Layout::right_to_left(egui::Align::Center);
        let builder = UiBuilder::new().id_salt(id).max_rect(rect).layout(layout);
        let mut child = ui.new_child(builder);
        let clicked = match self {
            Trailing::Button { label, icon } => {
                let mut button = Button::secondary(label).size(Size::Small);
                if let Some(icon) = icon {
                    button = button.icon(icon);
                }
                child.add(button).clicked()
            }
            Trailing::Icon { icon, name } => icon_button(&mut child, icon, name),
        };
        TrailingResponse { clicked }
    }
}

fn icon_button(ui: &mut Ui, icon: Icon, name: &str) -> bool {
    let c = theme::colors(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(Vec2::splat(metrics::CONTROL_SM), Sense::click());
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, name));
    if response.hovered() {
        ui.painter()
            .rect_filled(rect, CornerRadius::same(metrics::RADIUS_MD), c.bg_hover);
    }
    let ink = if response.hovered() { c.fg } else { c.fg_muted };
    let glyph = text::whole(ui, icon.rich(metrics::ICON_SM, ink));
    ui.painter()
        .galley(rect.center() - glyph.size() / 2.0, glyph, ink);
    focus::ring(ui, &response, rect, metrics::RADIUS_MD);
    response.on_hover_text(name).clicked()
}
