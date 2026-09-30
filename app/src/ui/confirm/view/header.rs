//! The header (`docs/ux.md` §4.2, §4.3, §4.3.1): eyebrow with the
//! permission chip, who asks, what they want, and the caller's warnings.
//!
//! Screen readers get the whole header as one sentence on the caller line
//! ("Signature request, app.diagnos.health, via Chrome, Allowed site"), so
//! the site is announced with its status and warnings together (§14).

use egui::{Align, Layout, Ui, UiBuilder, Vec2};
use websign_i18n::{Catalog, k};
use websign_ui_model::confirm::port::{CallerView, Mode};

use super::{Screen, origin};
use crate::ui::confirm::words;
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::chip::Chip;
use crate::ui::widgets::text;
use crate::ui::widgets::tone::Tone;

const EYEBROW_HEIGHT: f32 = 22.0;
const EYEBROW_GAP: f32 = 6.0;

pub fn show(ui: &mut Ui, s: &mut Screen<'_>) {
    ui.spacing_mut().item_spacing = Vec2::ZERO;
    let (lead, icon) = eyebrow(s);
    let chip = chip(s.tr, &s.view.header.caller, s.view.header.remembered);
    eyebrow_row(ui, &lead, icon, &chip);
    ui.add_space(metrics::SPACE_1);
    let asks = match s.view.mode {
        Mode::Sign { .. } => s.tr.tr(k::CONFIRM_ASKS_SIGN),
        Mode::Choose => s.tr.tr(k::CONFIRM_ASKS_SELECT),
    }
    .to_string();
    let spoken = format!("{lead}, {}, {}", words::site(&s.view.header.caller), chip.1);
    origin::show(ui, s.tr, &s.view.header.caller, &asks, &spoken);
}

/// "Signature request · via Chrome" (or "1 of 3", or "from a program").
fn eyebrow(s: &Screen<'_>) -> (String, Icon) {
    let tr = s.tr;
    let (title, icon) = match (s.view.mode, s.view.header.queue) {
        (Mode::Choose, _) => (
            tr.tr(k::CONFIRM_EYEBROW_SELECT).to_string(),
            icons::CERTIFICATE,
        ),
        (Mode::Sign { .. }, Some((current, total))) => (
            tr.tr(k::CONFIRM_EYEBROW_QUEUE)
                .arg("current", current)
                .arg("total", total)
                .to_string(),
            icons::SIGNATURE_REQUEST,
        ),
        (Mode::Sign { .. }, None) => (
            tr.tr(k::CONFIRM_EYEBROW).to_string(),
            icons::SIGNATURE_REQUEST,
        ),
    };
    let via = match &s.view.header.caller {
        CallerView::Web { browser, .. } => words::browser(*browser).map(|name| {
            tr.tr(k::CONFIRM_VIA_BROWSER)
                .arg("browser", name)
                .to_string()
        }),
        CallerView::Desktop { .. } => Some(tr.tr(k::CONFIRM_VIA_APP).to_string()),
    };
    let lead = match via {
        Some(via) => format!("{title} · {via}"),
        None => title,
    };
    (lead, icon)
}

/// The permission chip: `(text, accessible name, tone)`.
struct ChipText(String, String, Tone);

fn chip(tr: &Catalog, caller: &CallerView, remembered: bool) -> ChipText {
    let web = matches!(caller, CallerView::Web { .. });
    let (label, spoken) = match (web, remembered) {
        (true, true) => (k::CONFIRM_SITE_REMEMBERED, k::CONFIRM_SITE_REMEMBERED),
        (true, false) => (k::CONFIRM_SITE_NEW, k::CONFIRM_SITE_NEW_A11Y),
        (false, true) => (k::CONFIRM_APP_REMEMBERED, k::CONFIRM_APP_REMEMBERED),
        (false, false) => (k::CONFIRM_APP_NEW, k::CONFIRM_APP_NEW_A11Y),
    };
    let tone = if remembered {
        Tone::Success
    } else {
        Tone::Neutral
    };
    ChipText(tr.tr(label).to_string(), tr.tr(spoken).to_string(), tone)
}

fn eyebrow_row(ui: &mut Ui, lead: &str, icon: Icon, chip: &ChipText) {
    let muted = theme::colors(ui.ctx()).fg_muted;
    let (row, _) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), EYEBROW_HEIGHT),
        egui::Sense::hover(),
    );
    let chip_widget = chip_widget(chip);
    let chip_rect = ui
        .scope_builder(
            UiBuilder::new()
                .max_rect(row)
                .layout(Layout::right_to_left(Align::Center)),
            |ui| ui.add(chip_widget).rect,
        )
        .inner;
    let glyph = text::whole(ui, icon.rich(metrics::ICON_SM, muted));
    let room = chip_rect.min.x - row.min.x - metrics::ICON_SM - EYEBROW_GAP - metrics::SPACE_2;
    let lead = text::line(ui, typography::CAPTION.rich(lead).color(muted), room);
    let middle = row.center().y;
    let painter = ui.painter();
    painter.galley(
        egui::pos2(row.min.x, middle - glyph.size().y / 2.0),
        glyph,
        muted,
    );
    let x = row.min.x + metrics::ICON_SM + EYEBROW_GAP;
    painter.galley(egui::pos2(x, middle - lead.size().y / 2.0), lead, muted);
}

fn chip_widget(chip: &ChipText) -> Chip<'_> {
    let widget = Chip::new(chip.2, &chip.0).accessible_name(&chip.1);
    if chip.2 == Tone::Success {
        widget.icon(icons::SUCCESS)
    } else {
        widget.icon(icons::INFO)
    }
}
