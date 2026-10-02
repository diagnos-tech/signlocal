//! Drawing one [`ConfirmView`] (`docs/ux.md` §4.2): a fixed header with who
//! asks, a fixed 64 px footer with the hint and the buttons, and the body
//! between them, which scrolls when space is short. Every part only reads
//! the view and reports what the person did as [`Action`]s.

mod body;
mod checkbox;
mod code;
mod details;
mod empty;
mod error;
pub(super) mod fit;
mod footer;
mod header;
mod list;
mod origin;
mod outcome;
mod pin;
mod possible;
mod remember;
mod rows;

use std::time::Instant;

use egui::{CentralPanel, Frame, Margin, Panel, ScrollArea, Ui};
use websign_i18n::Catalog;
use websign_ui_model::confirm::{ConfirmView, UserInput};

use super::session::Session;
use crate::ui::theme;

/// What the person did this frame.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Input(UserInput),
    OpenUrl(String),
    Copy(String),
}

/// Everything a part of the window needs to draw itself.
#[derive(Debug)]
pub struct Screen<'a> {
    pub view: &'a ConfirmView,
    pub tr: &'a Catalog,
    pub session: &'a mut Session,
    pub out: &'a mut Vec<Action>,
    pub now: Instant,
}

impl Screen<'_> {
    pub fn input(&mut self, input: UserInput) {
        self.out.push(Action::Input(input));
    }
}

/// Header `padding: 20px 24px 16px`; body `16px 24px 4px`; footer `0 24px`.
const HEADER_MARGIN: Margin = Margin {
    left: 24,
    right: 24,
    top: 20,
    bottom: 16,
};
/// The body's bottom padding is only the gap kept above the footer: it
/// scrolls with the content, so any more than the fit leaves (`fit.rs`)
/// would make the body scroll by a sliver and show its scrollbar.
pub(super) const BODY_MARGIN: Margin = Margin {
    left: 24,
    right: 24,
    top: 16,
    bottom: 4,
};
const FOOTER_MARGIN: Margin = Margin::symmetric(24, 0);
const FOOTER_HEIGHT: f32 = 64.0;

pub fn show(ui: &mut Ui, mut screen: Screen<'_>) {
    let c = theme::colors(ui.ctx());
    Panel::top("confirm.header")
        .frame(Frame::new().fill(c.bg_surface).inner_margin(HEADER_MARGIN))
        .show_separator_line(true)
        .show(ui, |ui| header::show(ui, &mut screen));
    Panel::bottom("confirm.footer")
        .exact_size(FOOTER_HEIGHT)
        .frame(Frame::new().fill(c.bg_surface).inner_margin(FOOTER_MARGIN))
        .show_separator_line(true)
        .show(ui, |ui| footer::show(ui, &mut screen));
    CentralPanel::no_frame()
        .frame(Frame::new().fill(c.bg_canvas))
        .show(ui, |ui| {
            let viewport = ui.available_height() - BODY_MARGIN.sum().y;
            screen.session.fit.set_viewport(viewport);
            ScrollArea::vertical()
                .auto_shrink([false, false])
                .show(ui, |ui| {
                    Frame::new().inner_margin(BODY_MARGIN).show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        body::show(ui, &mut screen);
                    });
                });
        });
}
