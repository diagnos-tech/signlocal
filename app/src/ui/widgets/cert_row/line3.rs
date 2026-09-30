//! Line 3 of a certificate row: where the key is (shrinks), its validity
//! or the reason it cannot sign (never cut, `docs/ux.md` §5.1) and, on the
//! selected row, "Details".

use egui::{Rect, Response, Ui, pos2};

use super::CertRow;
use crate::ui::theme::{self, typography};
use crate::ui::widgets::tone::Tone;
use crate::ui::widgets::{link, text};

/// Line-3 icons are 14 px, a notch under the 16 px text-line icons.
const LINE_ICON: f32 = 14.0;
/// Between the line-3 items (location, separator, validity), as `gap: 6px`.
const LINE_GAP: f32 = 6.0;
/// Between a line-3 icon and its text.
const ICON_GAP: f32 = 4.0;
/// Room kept between the validity and "Details".
const DETAILS_GAP: f32 = 8.0;

impl CertRow<'_> {
    /// Location (shrinks) · status (never cut) · Details (selected only).
    pub(super) fn paint_line3(
        &self,
        ui: &mut Ui,
        id: egui::Id,
        line: Rect,
        muted: egui::Color32,
    ) -> Option<Response> {
        let c = theme::colors(ui.ctx());
        let t = self.text;
        let status_color = if self.enabled || t.status_tone != Tone::Neutral {
            t.status_tone.palette(&c).text
        } else {
            muted
        };
        let status = text::whole(ui, typography::SMALL.rich(t.status).color(status_color));
        let status_icon = t
            .status_icon
            .map(|icon| text::whole(ui, icon.rich(LINE_ICON, status_color)));
        let details = self.details.filter(|_| self.selected || !self.radio);
        let details_width = details.map_or(0.0, |label| link::width(ui, label) + DETAILS_GAP);
        let separator = text::whole(ui, typography::SMALL.rich("·").color(c.fg_subtle));
        let icon_width = status_icon.as_ref().map_or(0.0, |_| LINE_ICON + ICON_GAP);
        let status_width = icon_width + status.size().x;

        let location_room = line.width()
            - details_width
            - status_width
            - separator.size().x
            - 2.0 * LINE_GAP
            - LINE_ICON
            - LINE_GAP;
        let location = text::line(
            ui,
            typography::SMALL.rich(t.location).color(muted),
            location_room,
        );
        let painter = ui.painter();
        let mut x = line.min.x;
        let glyph = text::whole(ui, t.location_icon.rich(LINE_ICON, muted));
        painter.galley(pos2(x, line.min.y), glyph, muted);
        x += LINE_ICON + LINE_GAP;
        let location_width = location.size().x;
        painter.galley(pos2(x, line.min.y), location, muted);
        x += location_width + LINE_GAP;
        let separator_width = separator.size().x;
        painter.galley(pos2(x, line.min.y), separator, c.fg_subtle);
        x += separator_width + LINE_GAP;
        if let Some(glyph) = status_icon {
            painter.galley(pos2(x, line.min.y), glyph, status_color);
            x += LINE_ICON + ICON_GAP;
        }
        painter.galley(pos2(x, line.min.y), status, status_color);

        let label = details?;
        let at = pos2(line.max.x - link::width(ui, label), line.min.y);
        Some(link::paint(ui, id, at, label))
    }
}
