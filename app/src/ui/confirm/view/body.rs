//! The body between header and footer: what the window shows in each state
//! of `docs/ux.md` §4.8, sections 16 px apart.

use egui::Ui;
use websign_ui_model::confirm::ConfirmState;
use websign_ui_model::confirm::port::Failure;

use super::fit::MustSee;
use super::{Screen, code, empty, error, list, outcome, pin, remember};
use crate::ui::theme::metrics;

pub fn show(ui: &mut Ui, s: &mut Screen<'_>) {
    ui.spacing_mut().item_spacing = egui::Vec2::ZERO;
    match s.view.state {
        ConfirmState::Idle => {}
        ConfirmState::Success | ConfirmState::SiteCancelled | ConfirmState::Timeout => {
            outcome::show(ui, s);
        }
        ConfirmState::LoadingCerts => empty::loading(ui, s),
        ConfirmState::Empty => empty::show(ui, s),
        _ => request(ui, s),
    }
}

/// Choosing, ready, signing and the recoverable states: the error notice
/// (above the list, so another certificate is one click away), the code
/// card, the list, the PIN and "Remember"; the notice, the PIN block and
/// "Remember" kept on screen (`fit.rs`).
fn request(ui: &mut Ui, s: &mut Screen<'_>) {
    let top = ui.cursor().top();
    let mut gap = false;
    let mut section = |ui: &mut Ui| {
        if gap {
            ui.add_space(metrics::SPACE_4);
        }
        gap = true;
    };
    let mut notice = None;
    // A locked PIN is told where the PIN field was (§4.6).
    if let Some(failure) = s
        .view
        .banner
        .clone()
        .filter(|f| !matches!(f, Failure::PinLocked { .. }))
    {
        section(ui);
        notice = Some(ui.scope(|ui| error::show(ui, s, &failure)).response.rect);
    }
    if code::visible(s.view) {
        section(ui);
        code::show(ui, s);
    }
    section(ui);
    let rows_box = list::show(ui, s);
    let (mut pin, mut field) = (None, None);
    if pin::visible(s.view) {
        section(ui);
        let block = ui.scope(|ui| pin::show(ui, s));
        (pin, field) = (Some(block.response.rect), block.inner);
    }
    let mut remember = None;
    if remember::visible(s.view) {
        section(ui);
        remember = remember::show(ui, s).map(|block| block.rect);
    }
    match rows_box {
        Some(rows_box) => {
            let shown = MustSee {
                notice,
                pin,
                field: field.as_ref(),
                remember,
            };
            s.session.fit.place(ui, top, rows_box, shown);
        }
        None => s.session.fit.nothing_shown(),
    }
}
