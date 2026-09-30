//! Notices (`docs/ux.md` §4.3 origin warnings, §4.6 PIN locked, §4.8 errors,
//! §4.10 "The site will receive…"): an icon, an optional title and a text
//! that wraps, on the tone's soft background.
//!
//! The whole notice is one accessible node whose name is title + text, with
//! a live setting from its tone, so a screen reader announces an error the
//! moment it appears (§14: errors assertive, the rest polite). Actions go
//! below it, laid out by the window.

use egui::accesskit::Role;
use egui::{CornerRadius, Response, Sense, Stroke, StrokeKind, Ui, Vec2, Widget, pos2};

use super::text;
use super::tone::Tone;
use crate::ui::icons::Icon;
use crate::ui::theme::{self, metrics, typography};

/// `.callout` (padding 10/12, gap 10) and `.c-alert` (6/10, gap 8).
const PADDING: Vec2 = Vec2::new(12.0, 10.0);
const COMPACT_PADDING: Vec2 = Vec2::new(10.0, 6.0);
const GAP: f32 = 10.0;
const COMPACT_GAP: f32 = 8.0;

/// A notice.
#[derive(Debug, Clone, Copy)]
pub struct Banner<'a> {
    tone: Tone,
    icon: Icon,
    title: Option<&'a str>,
    body: &'a str,
    compact: bool,
}

impl<'a> Banner<'a> {
    pub fn new(tone: Tone, body: &'a str) -> Self {
        Banner {
            tone,
            icon: tone.icon(),
            title: None,
            body,
            compact: false,
        }
    }

    pub fn title(mut self, title: &'a str) -> Self {
        self.title = Some(title);
        self
    }

    pub fn icon(mut self, icon: Icon) -> Self {
        self.icon = icon;
        self
    }

    /// The one-line origin warning under the header (§4.2): tighter padding
    /// and `radius-md`.
    pub fn compact(mut self) -> Self {
        self.compact = true;
        self
    }
}

impl Widget for Banner<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let c = theme::colors(ui.ctx());
        let palette = self.tone.palette(&c);
        // Info notices sit on the plain surface with muted text; the others
        // keep full-contrast text on their soft color.
        let ink = if self.tone == Tone::Info {
            c.fg_muted
        } else {
            c.fg
        };
        let padding = if self.compact {
            COMPACT_PADDING
        } else {
            PADDING
        };
        let gap = if self.compact { COMPACT_GAP } else { GAP };
        let text_width = ui.available_width() - 2.0 * padding.x - metrics::ICON_SM - gap;
        let title = self.title.map(|title| {
            text::wrapped(
                ui,
                typography::BODY_STRONG.rich(title).color(ink),
                text_width,
            )
        });
        let body = text::wrapped(ui, typography::SMALL.rich(self.body).color(ink), text_width);
        let title_height = title.as_ref().map_or(0.0, |galley| galley.size().y);
        let height = (title_height + body.size().y).max(metrics::ICON_SM) + 2.0 * padding.y;
        let (rect, response) =
            ui.allocate_exact_size(Vec2::new(ui.available_width(), height), Sense::hover());

        let name = match self.title {
            Some(title) => format!("{title}. {}", self.body),
            None => self.body.to_owned(),
        };
        let role = if self.tone == Tone::Danger {
            Role::Alert
        } else {
            Role::Status
        };
        ui.ctx().accesskit_node_builder(response.id, |node| {
            // egui files a label's text as its value; an alert is named.
            node.clear_value();
            node.set_label(name);
            node.set_role(role);
            node.set_live(self.tone.live());
        });

        let radius = if self.compact {
            metrics::RADIUS_MD
        } else {
            metrics::RADIUS_LG
        };
        let painter = ui.painter();
        painter.rect(
            rect,
            CornerRadius::same(radius),
            palette.fill,
            Stroke::new(1.0, palette.border),
            StrokeKind::Inside,
        );
        // The icon is centered on the first line (the title's, when any).
        let first_line = if title.is_some() {
            typography::BODY_STRONG.line_height
        } else {
            typography::SMALL.line_height
        };
        let glyph = text::whole(ui, self.icon.rich(metrics::ICON_SM, palette.icon));
        let icon_at = rect.min + padding + Vec2::new(0.0, (first_line - glyph.size().y) / 2.0);
        painter.galley(icon_at, glyph, palette.icon);
        let mut cursor = pos2(
            rect.min.x + padding.x + metrics::ICON_SM + gap,
            rect.min.y + padding.y,
        );
        if let Some(title) = title {
            cursor.y += title.size().y;
            painter.galley(pos2(cursor.x, cursor.y - title.size().y), title, ink);
        }
        painter.galley(cursor, body, ink);
        response
    }
}
