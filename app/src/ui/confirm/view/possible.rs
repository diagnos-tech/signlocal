//! Devices that look like they hold a certificate but brought none
//! (`docs/ux.md` §6.2): a card saying what to install, with the download
//! link for this OS and "I installed it, scan again"; in a list that has
//! certificates, a compact row that expands into the same card.

use egui::{Align, Layout, Rect, Response, Sense, Stroke, Ui, Vec2, WidgetInfo, WidgetType, pos2};
use websign_i18n::{Catalog, k};
use websign_ui_model::confirm::UserInput;
use websign_ui_model::possible::PossibleCard;

use super::{Action, Screen};
use crate::ui::confirm::words;
use crate::ui::icons::{self, Icon};
use crate::ui::theme::{self, metrics, typography};
use crate::ui::widgets::button::{Button, Size};
use crate::ui::widgets::{focus, text};

const BUBBLE: f32 = 32.0;
const BUBBLE_ICON: f32 = 18.0;
const CARD_PADDING: Vec2 = Vec2::new(16.0, 14.0);

/// The rows of the non-empty list, one per device, each expandable.
pub fn inline(ui: &mut Ui, s: &mut Screen<'_>) {
    let count = s.view.possible.len();
    for index in 0..count {
        let card = s.view.possible[index].clone();
        let open = s.session.possible_open == Some(index);
        let title =
            s.tr.tr(k::POSSIBLE_INLINE)
                .arg("device", device(s.tr, &card))
                .to_string();
        let action = s.tr.tr(k::POSSIBLE_INLINE_ACTION).to_string();
        let last = index + 1 == count && !open;
        if compact_row(ui, icons::TOKEN, &title, Some(&action), last).clicked() {
            s.session.possible_open = if open { None } else { Some(index) };
        }
        if open {
            divider(ui);
            show_card(ui, s, &card);
        }
    }
}

/// A 40 px row with an icon, a caption and an optional link-like action
/// on the right; the whole row is one button.
pub fn compact_row(
    ui: &mut Ui,
    icon: Icon,
    title: &str,
    action: Option<&str>,
    last: bool,
) -> Response {
    let c = theme::colors(ui.ctx());
    let (rect, response) = ui.allocate_exact_size(
        Vec2::new(ui.available_width(), metrics::ROW_COMPACT),
        Sense::click(),
    );
    response.widget_info(|| WidgetInfo::labeled(WidgetType::Button, true, title));
    if response.hovered() {
        let radius = if last {
            crate::ui::widgets::list::Position {
                first: false,
                last: true,
            }
            .corners()
        } else {
            egui::CornerRadius::ZERO
        };
        ui.painter().rect_filled(rect, radius, c.bg_hover);
    }
    ui.painter()
        .hline(rect.x_range(), rect.min.y + 0.5, Stroke::new(1.0, c.border));
    let muted = c.fg_muted;
    let glyph = text::whole(ui, icon.rich(metrics::ICON_SM, muted));
    let x = rect.min.x + metrics::SPACE_4;
    ui.painter().galley(
        pos2(x, rect.center().y - glyph.size().y / 2.0),
        glyph,
        muted,
    );
    let mut right = rect.max.x - metrics::SPACE_4;
    if let Some(action) = action {
        let link = text::whole(ui, typography::CAPTION.rich(action).color(c.accent_fg));
        right -= link.size().x;
        ui.painter().galley(
            pos2(right, rect.center().y - link.size().y / 2.0),
            link,
            c.accent_fg,
        );
        right -= metrics::SPACE_2;
    }
    let left = x + metrics::ICON_SM + metrics::SPACE_2;
    let label = text::line(
        ui,
        typography::CAPTION.rich(title).color(muted),
        right - left,
    );
    ui.painter().galley(
        pos2(left, rect.center().y - label.size().y / 2.0),
        label,
        muted,
    );
    focus::ring_inside(ui, &response, rect, egui::CornerRadius::ZERO);
    response
}

/// The actionable card: warning bubble, title, what to install, buttons.
pub fn show_card(ui: &mut Ui, s: &mut Screen<'_>, card: &PossibleCard) {
    let c = theme::colors(ui.ctx());
    let tr = s.tr;
    let title = tr
        .tr(k::POSSIBLE_TITLE)
        .arg("device", device(tr, card))
        .to_string();
    let body = body(tr, card);
    egui::Frame::new()
        .inner_margin(egui::Margin::symmetric(
            CARD_PADDING.x as i8,
            CARD_PADDING.y as i8,
        ))
        .show(ui, |ui| {
            ui.horizontal_top(|ui| {
                ui.spacing_mut().item_spacing = Vec2::new(metrics::SPACE_3, 0.0);
                let (bubble, _) = ui.allocate_exact_size(Vec2::splat(BUBBLE), Sense::hover());
                ui.painter()
                    .circle_filled(bubble.center(), BUBBLE / 2.0, c.warning_soft);
                let glyph = text::whole(ui, icons::TOKEN.rich(BUBBLE_ICON, c.warning));
                ui.painter()
                    .galley(bubble.center() - glyph.size() / 2.0, glyph, c.warning);
                ui.vertical(|ui| {
                    ui.spacing_mut().item_spacing = Vec2::new(metrics::SPACE_2, 2.0);
                    ui.label(typography::BODY_STRONG.rich(&title).color(c.fg));
                    ui.label(typography::SMALL.rich(&body).color(c.fg_muted));
                    ui.add_space(metrics::SPACE_2);
                    actions(ui, s, card);
                });
            });
        });
}

fn actions(ui: &mut Ui, s: &mut Screen<'_>, card: &PossibleCard) {
    let tr = s.tr;
    let download = card.download.as_ref().map(|url| {
        (
            url.clone(),
            tr.tr(k::COMMON_DOWNLOAD_FOR)
                .arg("os", words::this_os(tr))
                .to_string(),
        )
    });
    let rescan = tr.tr(k::POSSIBLE_RESCAN).to_string();
    ui.with_layout(
        Layout::left_to_right(Align::Min).with_main_wrap(true),
        |ui| {
            if let Some((url, label)) = &download
                && Button::secondary(label)
                    .icon(icons::DOWNLOAD)
                    .size(Size::Medium)
                    .show(ui)
                    .response
                    .clicked()
            {
                s.out.push(Action::OpenUrl(url.clone()));
            }
            if Button::ghost(&rescan)
                .icon(icons::SCAN_AGAIN)
                .show(ui)
                .response
                .clicked()
            {
                s.input(UserInput::Rescan);
            }
        },
    );
}

/// "SafeNet eToken 5110", or the reader an unknown card sits in.
fn device(tr: &Catalog, card: &PossibleCard) -> String {
    card.name
        .clone()
        .or_else(|| card.reader.clone())
        .unwrap_or_else(|| tr.tr(k::CERT_WHERE_UNKNOWN_HW).to_string())
}

fn body(tr: &Catalog, card: &PossibleCard) -> String {
    let install = match (&card.name, &card.driver) {
        (None, _) => tr
            .tr(k::POSSIBLE_BODY_UNKNOWN_CARD)
            .arg("reader", card.reader.as_deref().unwrap_or_default())
            .to_string(),
        (Some(_), Some(driver)) if card.card => tr
            .tr(k::POSSIBLE_BODY_DRIVER_CARD)
            .arg("driver", driver)
            .to_string(),
        (Some(_), Some(driver)) => tr
            .tr(k::POSSIBLE_BODY_DRIVER_TOKEN)
            .arg("driver", driver)
            .to_string(),
        (Some(_), None) => tr.tr(k::POSSIBLE_NO_LINK).to_string(),
    };
    if card.download.is_none() && card.name.is_some() && card.driver.is_some() {
        format!("{install} {}", tr.tr(k::POSSIBLE_NO_LINK))
    } else {
        install
    }
}

fn divider(ui: &mut Ui) {
    let c = theme::colors(ui.ctx());
    let rect: Rect = ui.available_rect_before_wrap();
    ui.painter()
        .hline(rect.x_range(), rect.min.y + 0.5, Stroke::new(1.0, c.border));
}
