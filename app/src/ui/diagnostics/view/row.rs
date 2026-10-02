//! A diagnostics row (`docs/ux.md` §8.3–8.4, the mockups' `.d-row`): at
//! least 56 px, padding 12/16, a 20 px leading icon, a title line (name and
//! a mono detail), a status line with a status icon, an optional extra line
//! (hint, path, ATR) and one trailing action.
//!
//! Painted by hand from measured text so the height is known before
//! anything is placed. The row is one AccessKit list item named by all its
//! lines; the action is its own button.

use std::sync::Arc;

use egui::accesskit::Role;
use egui::text::{Galley, LayoutJob};
use egui::{Rect, Sense, Stroke, Ui, WidgetInfo, WidgetType, pos2, vec2};

use super::action::{Trailing, TrailingResponse};
use crate::ui::icons::Icon;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::list::Position;
use crate::ui::widgets::text;
use crate::ui::widgets::tone::Tone;

use super::spans::{Span, Style, job};

const PADDING_X: f32 = 16.0;
const PADDING_Y: f32 = 12.0;
const GAP: f32 = 12.0;
const LINE_GAP: f32 = 2.0;
const STATUS_ICON: f32 = 14.0;
const STATUS_GAP: f32 = 6.0;

/// The status line: icon (colored by tone) and text.
#[derive(Debug, Clone)]
pub struct Status {
    /// `None` for plain metadata ("Remembered on …").
    pub icon: Option<Icon>,
    pub tone: Tone,
    pub text: String,
}

/// The line under the status.
#[derive(Debug, Clone)]
pub struct Extra {
    pub text: String,
    /// Paths and ATRs: mono, cut at the end instead of wrapped.
    pub mono: bool,
}

/// One row.
#[derive(Debug, Clone)]
pub struct Row<'a> {
    pub id: egui::Id,
    pub icon: Option<Icon>,
    pub title: Vec<Span>,
    pub status: Option<Status>,
    pub extra: Option<Extra>,
    pub trailing: Option<Trailing<'a>>,
    pub position: Position,
}

impl Row<'_> {
    /// Draws the row; `Some` when its action was used.
    pub fn show(self, ui: &mut Ui) -> TrailingResponse {
        let c = theme::colors(ui.ctx());
        let width = ui.available_width();
        let action_width = self.trailing.as_ref().map_or(0.0, |t| t.width(ui) + GAP);
        let left = PADDING_X + self.icon.map_or(0.0, |_| metrics::ICON_MD + GAP);
        let main = (width - left - PADDING_X - action_width).max(40.0);

        let title = ui
            .ctx()
            .fonts_mut(|fonts| fonts.layout_job(wrap(job(&self.title, &c), main)));
        let status = self.status.as_ref().map(|status| {
            let text = typography::SMALL.rich(&status.text).color(c.fg_muted);
            let lead = status.icon.map_or(0.0, |_| STATUS_ICON + STATUS_GAP);
            (text::wrapped(ui, text, main - lead), status)
        });
        let extra = self
            .extra
            .as_ref()
            .map(|extra| extra_galley(ui, extra, main, &c));
        let mut height = PADDING_Y * 2.0 + title.size().y;
        height += status.as_ref().map_or(0.0, |(g, _)| LINE_GAP + g.size().y);
        height += extra.as_ref().map_or(0.0, |g| LINE_GAP + g.size().y);
        let height = height.max(metrics::ROW_DIAG);

        let (rect, response) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
        let name = self.accessible_name();
        response.widget_info(|| WidgetInfo::labeled(WidgetType::Label, true, &name));
        ui.ctx().accesskit_node_builder(response.id, |node| {
            node.clear_value();
            node.set_label(name.clone());
            node.set_role(Role::ListItem);
        });
        let painter = ui.painter();
        if self.position.divider() {
            painter.hline(rect.x_range(), rect.min.y + 0.5, Stroke::new(1.0, c.border));
        }
        if let Some(icon) = self.icon {
            let glyph = text::whole(ui, icon.rich(metrics::ICON_MD, c.fg_muted));
            painter.galley(
                pos2(rect.min.x + PADDING_X, rect.min.y + PADDING_Y),
                glyph,
                c.fg_muted,
            );
        }
        let content_height = height - 2.0 * PADDING_Y;
        let natural = title.size().y
            + status.as_ref().map_or(0.0, |(g, _)| LINE_GAP + g.size().y)
            + extra.as_ref().map_or(0.0, |g| LINE_GAP + g.size().y);
        let x = rect.min.x + left;
        let mut y = rect.min.y + PADDING_Y + (content_height - natural) / 2.0;
        painter.galley(pos2(x, y), title.clone(), c.fg);
        y += title.size().y;
        if let Some((galley, status)) = status {
            y += LINE_GAP;
            let mut text_x = x;
            if let Some(icon) = status.icon {
                let color = status.tone.palette(&c).icon;
                let glyph = text::whole(ui, icon.rich(STATUS_ICON, color));
                let line = typography::SMALL.line_height;
                painter.galley(pos2(x, y + (line - glyph.size().y) / 2.0), glyph, color);
                text_x += STATUS_ICON + STATUS_GAP;
            }
            let size = galley.size();
            painter.galley(pos2(text_x, y), galley, c.fg_muted);
            y += size.y;
        }
        if let Some(galley) = extra {
            y += LINE_GAP;
            painter.galley(pos2(x, y), galley, c.fg_subtle);
        }
        match self.trailing {
            Some(trailing) => {
                let size = vec2(action_width - GAP, rect.height());
                let at =
                    Rect::from_min_size(pos2(rect.max.x - PADDING_X - size.x, rect.min.y), size);
                trailing.show(ui, at, self.id)
            }
            None => TrailingResponse::default(),
        }
    }

    /// Every line, for screen readers.
    fn accessible_name(&self) -> String {
        let title: String = self
            .title
            .iter()
            .map(|span| match span.style {
                Style::Mono => format!(" {}", span.text),
                _ => span.text.clone(),
            })
            .collect();
        let mut parts = vec![title];
        parts.extend(self.status.as_ref().map(|status| status.text.clone()));
        parts.extend(self.extra.as_ref().map(|extra| extra.text.clone()));
        parts.join(", ")
    }
}

/// One line, cut with "…": a name never pushes the status down.
fn wrap(mut job: LayoutJob, width: f32) -> LayoutJob {
    job.wrap.max_width = width;
    job.wrap.max_rows = 1;
    job.wrap.break_anywhere = true;
    job.wrap.overflow_character = Some('…');
    job
}

fn extra_galley(ui: &Ui, extra: &Extra, width: f32, c: &theme::Colors) -> Arc<Galley> {
    if extra.mono {
        text::line(
            ui,
            typography::MONO.rich(&extra.text).color(c.fg_subtle),
            width,
        )
    } else {
        text::wrapped(
            ui,
            typography::SMALL.rich(&extra.text).color(c.fg_subtle),
            width,
        )
    }
}
