//! A certificate row (`docs/ux.md` §5.1), shared by the confirmation list
//! (with the selection circle) and Diagnostics › Certificates (without).
//!
//! 72 px: padding 10/16, lines of 20 + 16 + 16. Line 1 holder name and type
//! badge; line 2 document and issuer; line 3 where the key is and its
//! validity. On line 3 the validity is never cut, the location shrinks
//! instead: validity is state and decides the choice (§5.1).
//!
//! The whole row is the click target and one accessible radio button whose
//! name is the full sentence the caller builds (§14), so truncated text
//! loses nothing for screen readers. Disabled rows stay focusable, to be read.
//! Space and a click select; Enter never does ([`super::keys`]): it is
//! reported apart so the window can move focus on (§4.9).

use egui::accesskit::Role;
use egui::{Rect, Response, Sense, Ui, Vec2, WidgetInfo, WidgetType, pos2, vec2};

use super::badge::Badge;
use super::list::Position;
use super::tone::Tone;
use super::{focus, keys, radio, text};

mod line3;
use crate::ui::icons::Icon;
use crate::ui::theme::{self, metrics, typography};

const PADDING: Vec2 = Vec2::new(16.0, 10.0);
const RADIO_GAP: f32 = 12.0;
/// The texts of one row, already localized and formatted.
#[derive(Debug, Clone, Copy)]
pub struct CertRowText<'a> {
    pub name: &'a str,
    pub badge: &'a str,
    /// "CPF •••.456.789-•• · AC SOLUTI Multipla v5".
    pub detail: &'a str,
    pub location_icon: Icon,
    pub location: &'a str,
    /// Validity, or the reason a disabled row cannot sign.
    pub status: &'a str,
    pub status_tone: Tone,
    pub status_icon: Option<Icon>,
    /// The full sentence for screen readers (§14).
    pub accessible_name: &'a str,
}

/// A row and how it is shown.
#[derive(Debug, Clone, Copy)]
pub struct CertRow<'a> {
    pub text: CertRowText<'a>,
    pub selected: bool,
    pub enabled: bool,
    /// Draw the selection circle and expose radio semantics. Off for a list
    /// of one (§4.5) and in Diagnostics.
    pub radio: bool,
    /// "Details" link label, shown on the selected (or radio-less) row.
    pub details: Option<&'a str>,
    /// Where the row sits in its [`super::list`]: divider and corners.
    pub position: Position,
}

/// What the person did with the row.
#[derive(Debug)]
pub struct CertRowResponse {
    /// `clicked()`: a click or Space (select). Never Enter.
    pub row: Response,
    /// Enter while the row had focus: move focus on, never sign (§4.9).
    pub enter: bool,
    pub details: Option<Response>,
}

impl CertRow<'_> {
    pub fn show(self, ui: &mut Ui) -> CertRowResponse {
        let c = theme::colors(ui.ctx());
        let size = vec2(ui.available_width(), metrics::ROW_CERT);
        let sense = if self.enabled {
            Sense::click()
        } else {
            Sense::focusable_noninteractive()
        };
        let (id, rect) = ui.allocate_space(size);
        let enter = keys::take_enter(ui, id);
        let row = ui.interact(rect, id, sense);
        let t = self.text;
        let (typ, role) = if self.radio {
            (WidgetType::RadioButton, Role::RadioButton)
        } else {
            (WidgetType::Other, Role::ListItem)
        };
        row.widget_info(|| {
            if self.radio {
                WidgetInfo::selected(typ, self.enabled, self.selected, t.accessible_name)
            } else {
                WidgetInfo::labeled(typ, self.enabled, t.accessible_name)
            }
        });
        ui.ctx()
            .accesskit_node_builder(row.id, |node| node.set_role(role));

        let fill = if self.selected {
            c.accent_soft
        } else if self.enabled && row.hovered() {
            c.bg_hover
        } else {
            egui::Color32::TRANSPARENT
        };
        let corners = self.position.corners();
        ui.painter().rect_filled(rect, corners, fill);
        if self.position.divider() {
            ui.painter().hline(
                rect.x_range(),
                rect.min.y + 0.5,
                egui::Stroke::new(1.0, c.border),
            );
        }

        let mut left = rect.min.x + PADDING.x;
        if self.radio {
            if self.enabled {
                let center = pos2(left + radio::SIZE / 2.0, rect.min.y + PADDING.y + 10.0);
                radio::paint(ui.painter(), center, self.selected, &c);
            }
            left += radio::SIZE + RADIO_GAP;
        }
        let right = rect.max.x - PADDING.x;
        let (strong, muted) = if self.enabled {
            (c.fg, c.fg_muted)
        } else {
            (c.fg_subtle, c.fg_subtle)
        };
        let top = rect.min.y + PADDING.y;

        let (badge, badge_size) = Badge::new(t.badge).measure(ui);
        // Centered on the 20 px name line, as `align-items: center`.
        let badge_top = top + (typography::BODY_STRONG.line_height - badge_size.y) / 2.0;
        Badge::paint(ui, pos2(right - badge_size.x, badge_top), badge, badge_size);
        let name_width = right - left - badge_size.x - metrics::SPACE_2;
        let name = text::line(
            ui,
            typography::BODY_STRONG.rich(t.name).color(strong),
            name_width,
        );
        let elided = name.elided;
        ui.painter().galley(pos2(left, top), name, strong);

        let detail = text::line(
            ui,
            typography::SMALL.rich(t.detail).color(muted),
            right - left,
        );
        ui.painter().galley(pos2(left, top + 20.0), detail, muted);

        let line3 = Rect::from_min_max(pos2(left, top + 36.0), pos2(right, top + 52.0));
        let details = self.paint_line3(ui, row.id.with("details"), line3, muted);
        focus::ring_inside(ui, &row, rect, corners);
        let row = if elided {
            row.on_hover_text(t.name)
        } else {
            row
        };
        CertRowResponse {
            row,
            enter,
            details,
        }
    }
}
