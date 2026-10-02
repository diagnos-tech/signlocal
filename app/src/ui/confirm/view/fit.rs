//! Keeping the body readable at a glance (`docs/ux.md` §4.5, §4.6, §4.10):
//! its top (the code card, the Continue hint or what Choose mode shares,
//! which say what the primary button does), the error notice, the whole PIN
//! block, which the window waits for, and the "Remember" block, whose help
//! text says what ticking it grants. At 480 × 600 a long list and a tall
//! header can push them out of view. The list gives way instead: its scroll
//! box shrinks to one row and then below it (always ending between two
//! lines of a row), so the list scrolls inside itself while the body stays
//! still. Only when not even half a row fits (an
//! error notice and the code card above the list) does the body scroll to
//! them, each time one of them changes (and to the field when it gets focus
//! out of view).
//!
//! Heights come from the previous pass: the body measures how much it needs
//! around the rows, and a pass whose measure changed is discarded, so the
//! person never sees the blocks jump.

use egui::style::ScrollAnimation;
use egui::{Align, Context, Rect, Response, Ui};

use crate::ui::theme::metrics;

/// A shortened box hides at least this much of a row, so what is left of
/// it reads as "more below" rather than as a clipping mistake.
const CUT_AT_LEAST: f32 = 0.4 * metrics::ROW_CERT;
/// A shortened box shows at least this much of the row after its last
/// whole one: its top and the start of its name, which read as "more
/// below"; less reads as a clipping mistake.
const PEEK_AT_LEAST: f32 = 16.0;
/// Where the name, detail and third lines of a certificate row end
/// (`widgets/cert_row`). Past the name line, a box ends between two lines:
/// a cut through the detail or the third line reads as a mistake.
const LINE_ENDS: [f32; 3] = [30.0, 46.0, 62.0];
/// The shortest box: one row's name line; under this the list is no longer
/// usable.
const SHORTEST: f32 = LINE_ENDS[0];

/// `height` shortened so that the row after the last whole one shows only
/// a peek of its name or ends between two of its lines.
fn between_lines(height: f32) -> f32 {
    let partial = height % metrics::ROW_CERT;
    let kept = if partial < PEEK_AT_LEAST {
        0.0
    } else if partial < LINE_ENDS[0] {
        partial
    } else {
        LINE_ENDS
            .into_iter()
            .rev()
            .find(|end| partial >= *end)
            .unwrap_or(0.0)
    };
    height - partial + kept
}

/// What the body measured for the request on screen (`Session::fit`).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Fit {
    /// Visible height of the body's content, inside its margins (the bottom
    /// one is the gap kept above the footer).
    viewport: f32,
    /// Height from the body's top to the bottom of the last block that must
    /// be seen, without the rows' scroll box; `None` while none is shown.
    around_rows: Option<f32>,
    /// That measure included our PIN field.
    with_field: bool,
    /// The blocks were brought into view since they last moved.
    revealed: bool,
}

/// What must be seen this pass, as drawn, each when shown: the error
/// notice, the PIN block (label, field or its stand-in, and the line under
/// it), our field in it, and the "Remember" block.
#[derive(Debug, Clone, Copy, Default)]
pub struct MustSee<'a> {
    pub notice: Option<Rect>,
    pub pin: Option<Rect>,
    pub field: Option<&'a Response>,
    pub remember: Option<Rect>,
}

impl MustSee<'_> {
    /// All the blocks as one rect; `None` when none is shown.
    fn rect(&self) -> Option<Rect> {
        [self.notice, self.pin, self.remember]
            .into_iter()
            .flatten()
            .reduce(Rect::union)
    }
}

impl Fit {
    /// The body's visible height inside its margins, this frame.
    pub fn set_viewport(&mut self, height: f32) {
        self.viewport = height;
    }

    /// Height of the rows' scroll box: `wanted`, or less so the whole body
    /// fits without scrolling; never under [`SHORTEST`].
    pub fn rows_height(&self, wanted: f32) -> f32 {
        let Some(around) = self.around_rows else {
            return wanted;
        };
        let room = self.viewport - around;
        if room >= wanted {
            return wanted;
        }
        between_lines(room.min(wanted - CUT_AT_LEAST)).max(SHORTEST)
    }

    /// Measures this pass from the body's `top` and the rows' box height,
    /// then scrolls to blocks that could not fit once the layout settles
    /// again: when they appear and whenever one of them changes (a notice,
    /// a PIN error line). A pass whose measure changed is discarded.
    pub fn place(&mut self, ui: &Ui, top: f32, rows_box: f32, shown: MustSee<'_>) {
        let Some(rect) = shown.rect() else {
            self.nothing_shown();
            return;
        };
        let around_rows = rect.bottom() - top - rows_box;
        let settled = self.measured(ui.ctx(), around_rows, shown.field.is_some());
        let first = settled && !self.revealed;
        let clip = ui.clip_rect();
        if first && !clip.contains_rect(rect) {
            // Taller than the body: the blocks one acts in (the PIN block,
            // "Remember") win over the top of the notice.
            let align = (rect.height() > clip.height()).then_some(Align::Max);
            ui.scroll_to_rect_animation(rect, align, ScrollAnimation::none());
        } else if let Some(field) = shown.field
            && field.gained_focus()
            && !clip.contains_rect(field.rect)
        {
            field.scroll_to_me(None);
        }
        self.revealed |= settled;
    }

    /// Records this pass's measure; returns whether the layout is settled.
    fn measured(&mut self, ctx: &Context, around_rows: f32, with_field: bool) -> bool {
        let settled = with_field == self.with_field
            && self
                .around_rows
                .is_some_and(|before| (before - around_rows).abs() < 0.5);
        if !settled {
            self.around_rows = Some(around_rows);
            self.with_field = with_field;
            self.revealed = false;
            ctx.request_discard("a block that must be seen moved");
        }
        settled
    }

    /// Whether our field has been measured: before that its place is not
    /// final, and focusing it would scroll the body (a text field scrolls
    /// its cursor into view).
    pub fn placed(&self) -> bool {
        self.around_rows.is_some() && self.with_field
    }

    /// Nothing to keep on screen: the next block is measured and revealed
    /// afresh.
    pub fn nothing_shown(&mut self) {
        *self = Fit {
            viewport: self.viewport,
            ..Fit::default()
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_partial_row_ends_between_two_of_its_lines() {
        let row = metrics::ROW_CERT;
        assert_eq!(between_lines(50.0), 46.0, "not through the third line");
        assert_eq!(
            between_lines(row + 10.0),
            row,
            "not a sliver of the next row"
        );
        assert_eq!(between_lines(row + 40.0), row + 30.0);
        assert_eq!(
            between_lines(row + 23.0),
            row + 23.0,
            "a peek of the next name"
        );
        assert_eq!(between_lines(2.0 * row), 2.0 * row);
        assert_eq!(between_lines(row + 65.0), row + 62.0);
    }
}
