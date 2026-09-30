//! "Sign with" and the certificate list (`docs/ux.md` §4.5, §5): the rows
//! that can sign and the collapsed "Can't sign (n)" group, in one scroll box
//! (three rows and the group at most; with our PIN field two; shorter so
//! the rest of the body fits — `fit.rs`), and the devices that brought no
//! certificate (§6.2). A filter appears above long lists (§5.13).

use egui::{ScrollArea, Ui, Vec2, WidgetInfo, WidgetType};
use websign_i18n::k;
use websign_ui_model::certs::{FILTER_THRESHOLD, RowStatus, matches_filter};
use websign_ui_model::confirm::port::Mode;

use super::{Screen, pin, possible, rows};
use crate::ui::icons;
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::list::{self, Position};
use crate::ui::widgets::text_field::TextField;

const LABEL_GAP: f32 = 8.0;

/// Draws the list; returns the height of the rows' scroll box.
pub fn show(ui: &mut Ui, s: &mut Screen<'_>) -> Option<f32> {
    let list = s.view.list.clone()?;
    let c = theme::colors(ui.ctx());
    let label = match s.view.mode {
        Mode::Sign { .. } => s.tr.tr(k::CERTS_LABEL_SIGN),
        Mode::Choose => s.tr.tr(k::CERTS_LABEL_SELECT),
    }
    .to_string();
    ui.label(typography::CAPTION.rich(&label).color(c.fg_muted));
    ui.add_space(LABEL_GAP);
    let usable_count = list.usable.len();
    if usable_count > FILTER_THRESHOLD {
        let placeholder = s.tr.tr(k::CERTS_FILTER_PLACEHOLDER).to_string();
        TextField::new(&mut s.session.filter, &placeholder)
            .hint(&placeholder)
            .icon(icons::FILTER)
            .show(ui);
        ui.add_space(LABEL_GAP);
    }
    let query = s.session.filter.clone();
    let shown: Vec<_> = list
        .usable
        .iter()
        .filter(|row| matches_filter(row, &query))
        .collect();
    let order: Vec<_> = shown
        .iter()
        .filter(|row| row.status == RowStatus::Usable)
        .map(|row| row.candidate.fingerprint)
        .collect();
    let radio = usable_count > 1;
    let rows = if pin::visible(s.view) { 2.0 } else { 3.0 };
    // "Can't sign (n)" scrolls with the rows, so a short box gives up the
    // disclosure before it gives up a row that can sign.
    let disclosure = if list.disabled.is_empty() {
        0.0
    } else {
        metrics::ROW_COMPACT
    };
    let rows_height = s
        .session
        .fit
        .rows_height(rows * metrics::ROW_CERT + disclosure);
    let trailing = !list.disabled.is_empty() || !s.view.possible.is_empty();
    let mut rows_box = 0.0;
    let group = list::show(ui, |ui| {
        ui.set_width(ui.available_width());
        rows_box = ScrollArea::vertical()
            .id_salt("confirm.rows")
            .max_height(rows_height)
            .min_scrolled_height(rows_height)
            .auto_shrink([false, true])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = Vec2::ZERO;
                if shown.is_empty() && !query.trim().is_empty() {
                    no_match(ui, s, &query);
                }
                for (index, row) in shown.iter().enumerate() {
                    let mut position = Position::of(index, shown.len());
                    position.last &= !trailing;
                    rows::show(ui, s, row, radio, position, &order);
                }
                if !list.disabled.is_empty() {
                    disabled_group(ui, s, &list.disabled, !s.view.possible.is_empty());
                }
            })
            .inner_rect
            .height();
        possible::inline(ui, s);
    });
    group
        .response
        .widget_info(|| WidgetInfo::labeled(WidgetType::RadioGroup, true, &label));
    Some(rows_box)
}

/// "Can't sign (n)", collapsed by default; expanded, the rows it hides.
fn disabled_group(
    ui: &mut Ui,
    s: &mut Screen<'_>,
    disabled: &[websign_ui_model::certs::CertRow],
    more_below: bool,
) {
    let title =
        s.tr.tr(k::CERTS_UNUSABLE_GROUP)
            .arg("count", disabled.len())
            .to_string();
    let icon = if s.session.disabled_open {
        icons::COLLAPSE
    } else {
        icons::EXPAND
    };
    let last = !s.session.disabled_open && !more_below;
    if possible::compact_row(ui, icon, &title, None, last).clicked() {
        s.session.disabled_open = !s.session.disabled_open;
    }
    if s.session.disabled_open {
        for (index, row) in disabled.iter().enumerate() {
            let mut position = Position::of(index + 1, disabled.len() + 1);
            position.last &= !more_below;
            rows::show(ui, s, row, true, position, &[]);
        }
    }
}

fn no_match(ui: &mut Ui, s: &Screen<'_>, query: &str) {
    let c = theme::colors(ui.ctx());
    let text =
        s.tr.tr(k::CERTS_FILTER_EMPTY)
            .arg("query", query)
            .to_string();
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(16, 12))
        .show(ui, |ui| {
            ui.label(typography::SMALL.rich(text).color(c.fg_muted))
        });
}
