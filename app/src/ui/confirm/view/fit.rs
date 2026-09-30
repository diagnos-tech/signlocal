//! Keeping our PIN field on screen without scrolling (`docs/ux.md` §4.6): at
//! 480 × 600 the code card, a long list and a site notice can push the field
//! below the footer, where the person never sees what the window waits for.
//! The rows' scroll box gives up height first, down to one row, so the field
//! fits; when even that is not enough, the body scrolls to the field once,
//! when it appears (or when it gets focus out of view).
//!
//! Heights come from the previous pass: the body measures how much it needs
//! around the rows, and a pass whose measure changed is discarded, so the
//! person never sees the field jump.

use egui::{Context, Response, Ui};

use crate::ui::theme::metrics;

/// Space kept between the field and the footer.
const BELOW_FIELD: f32 = metrics::SPACE_1;
/// A shortened box hides at least this much of a row, so what is left of
/// it reads as "more below" rather than as a clipping mistake.
const CUT_AT_LEAST: f32 = 0.4 * metrics::ROW_CERT;

/// What the body measured for the request on screen (`Session::fit`).
#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Fit {
    /// Visible height of the body's content, below its top margin.
    viewport: f32,
    /// Height from the body's top to the PIN field's bottom, without the
    /// rows' scroll box; `None` while no field is shown.
    around_rows: Option<f32>,
    /// The field was brought into view since it appeared.
    revealed: bool,
}

impl Fit {
    /// The body's visible height below its top margin, this frame.
    pub fn set_viewport(&mut self, height: f32) {
        self.viewport = height;
    }

    /// Height of the rows' scroll box: `wanted`, or less so the field fits.
    /// Never under one row; when not even that makes room (an error notice
    /// above the list), the rows keep their height and the body scrolls.
    pub fn rows_height(&self, wanted: f32) -> f32 {
        let Some(around) = self.around_rows else {
            return wanted;
        };
        let room = self.viewport - around - BELOW_FIELD;
        if room >= wanted || room < metrics::ROW_CERT.min(wanted) {
            return wanted;
        }
        room.min(wanted - CUT_AT_LEAST)
            .max(metrics::ROW_CERT.min(wanted))
    }

    /// Records this pass's measure; a changed one discards the pass.
    /// Returns whether the layout is settled.
    pub fn measured(&mut self, ctx: &Context, around_rows: f32) -> bool {
        let settled = self
            .around_rows
            .is_some_and(|before| (before - around_rows).abs() < 0.5);
        if !settled {
            self.around_rows = Some(around_rows);
            ctx.request_discard("the PIN field moved");
        }
        settled
    }

    /// Whether the field has been measured: before that its place is not
    /// final, and focusing it would scroll the body (a text field scrolls
    /// its cursor into view).
    pub fn placed(&self) -> bool {
        self.around_rows.is_some()
    }

    /// Once the layout is settled, scrolls the body to a field it could not
    /// fit (or to a field that gets focus out of view).
    pub fn reveal(&mut self, ui: &Ui, field: &Response, settled: bool) {
        let first = settled && !self.revealed;
        if (first || field.gained_focus()) && !ui.clip_rect().contains_rect(field.rect) {
            field.scroll_to_me(None);
        }
        self.revealed |= settled;
    }

    /// No field on screen: the next one is measured and revealed afresh.
    pub fn no_field(&mut self) {
        self.around_rows = None;
        self.revealed = false;
    }
}
